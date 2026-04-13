//! iron_server — Main Chat Server
//!
//! Async TCP server with authentication, broadcast messaging,
//! and graceful shutdown using tokio::signal.

use anyhow::Result;
use iron_server::{demo_users, new_broadcast, Message, OnlineUsers, DEFAULT_ADDR};
use std::sync::Arc;
use tokio::{
    io::{AsyncWriteExt},
    net::{TcpListener, TcpStream},
    signal,
    sync::{broadcast, RwLock},
};
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("🚀 Starting iron_server on {}", DEFAULT_ADDR);

    let listener = TcpListener::bind(DEFAULT_ADDR).await?;
    let broadcast_tx = new_broadcast();
    let online_users: OnlineUsers = Arc::new(RwLock::new(HashMap::new()));
    let users_db = demo_users();

    info!("✅ iron_server is ready. Press Ctrl+C to shut down gracefully.");

    let mut shutdown = signal::ctrl_c();

    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, addr)) => {
                        info!("📥 New connection from {}", addr);
                        let tx = broadcast_tx.clone();
                        let online = online_users.clone();
                        let db = users_db.clone();

                        tokio::spawn(handle_client(stream, tx, online, db));
                    }
                    Err(e) => error!("Failed to accept connection: {}", e),
                }
            }

            _ = &mut shutdown => {
                info!("🛑 Shutdown signal received. Notifying all clients...");
                let _ = broadcast_tx.send(Message::Shutdown);
                break;
            }
        }
    }

    info!("👋 iron_server shut down gracefully.");
    Ok(())
}

async fn handle_client(
    stream: TcpStream,
    tx: broadcast::Sender<Message>,
    online_users: OnlineUsers,
    users_db: std::collections::HashMap<String, String>,
) -> Result<()> {
    let (read_half, mut write_half) = stream.into_split();
    let mut lines = FramedRead::new(read_half, LinesCodec::new());

    // Authentication
    let username = match authenticate(&mut lines, &users_db).await? {
        Some(u) => u,
        None => return Ok(()),
    };

    // Prevent duplicate usernames
    {
        let mut guard = online_users.write().await;
        if guard.contains_key(&username) {
            let _ = write_half.write_all(b"{\"type\":\"auth_error\",\"reason\":\"Username already taken\"}\n").await;
            return Ok(());
        }
        guard.insert(username.clone(), ());
    }

    // Join announcement
    let _ = tx.send(Message::UserJoined { username: username.clone() });
    let _ = tx.send(Message::OnlineUsers {
        usernames: online_users.read().await.keys().cloned().collect(),
    });

    info!("👤 {} joined the chat", username);

    // Write task
    let mut rx = tx.subscribe();
    let username_write = username.clone();
    let write_handle = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if matches!(msg, Message::Shutdown) {
                break;
            }
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = write_half.write_all((json + "\n").as_bytes()).await;
            }
        }
    });

    // Read loop
    while let Some(Ok(line)) = lines.next().await {
        let msg: Message = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(e) => {
                warn!("Invalid JSON from {}: {}", username, e);
                continue;
            }
        };

        if let Message::PublicMessage { content, .. } = msg {
            let _ = tx.send(Message::PublicMessage {
                username: username.clone(),
                content,
            });
        }
    }

    // Cleanup
    {
        let mut guard = online_users.write().await;
        guard.remove(&username);
    }
    let _ = tx.send(Message::UserLeft { username: username.clone() });
    info!("👤 {} left the chat", username);

    let _ = write_handle.await;
    Ok(())
}

async fn authenticate(
    lines: &mut FramedRead<tokio::io::ReadHalf<tokio::net::TcpStream>, LinesCodec>,
    users_db: &std::collections::HashMap<String, String>,
) -> Result<Option<String>> {
    let line = match lines.next().await {
        Some(Ok(l)) => l,
        _ => return Ok(None),
    };

    let msg: Message = match serde_json::from_str(&line) {
        Ok(m) => m,
        Err(_) => return Ok(None),
    };

    if let Message::Login { username, password } = msg {
        if let Some(stored) = users_db.get(&username) {
            if stored == &password {
                return Ok(Some(username));
            }
        }
    }

    Ok(None)
}