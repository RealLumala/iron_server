//! iron_server — CLI Chat Client

use anyhow::Result;
use iron_server::{Message, DEFAULT_ADDR};
use std::io::{self, Write};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::mpsc,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔌 iron_client — Connecting to iron_server...");

    let stream = TcpStream::connect(DEFAULT_ADDR).await?;
    let (read, mut write) = stream.into_split();

    let username = login(&mut write).await?;
    println!("✅ Welcome, {}!", username);
    println!("💬 Type your messages. Commands: /quit");

    let (input_tx, mut input_rx) = mpsc::channel::<String>(32);

    // Blocking stdin reader
    tokio::spawn(async move {
        let mut stdin = io::stdin().lock();
        loop {
            let mut line = String::new();
            if stdin.read_line(&mut line).is_err() {
                break;
            }
            let input = line.trim().to_string();
            if !input.is_empty() && input_tx.send(input).await.is_err() {
                break;
            }
        }
    });

    let mut reader = BufReader::new(read);
    let mut line_buf = String::new();

    loop {
        tokio::select! {
            read_result = reader.read_line(&mut line_buf) => {
                match read_result {
                    Ok(0) => { println!("\n🔌 Disconnected from server."); break; }
                    Ok(_) => {
                        if let Ok(msg) = serde_json::from_str::<Message>(&line_buf.trim()) {
                            match msg {
                                Message::PublicMessage { username, content } => {
                                    println!("{}: {}", username, content);
                                }
                                Message::UserJoined { username } => {
                                    println!("👋 {} joined", username);
                                }
                                Message::UserLeft { username } => {
                                    println!("👋 {} left", username);
                                }
                                Message::OnlineUsers { usernames } => {
                                    println!("👥 Online: {}", usernames.join(", "));
                                }
                                Message::Shutdown => {
                                    println!("\n🛑 Server shutting down...");
                                    break;
                                }
                                _ => {}
                            }
                        }
                        line_buf.clear();
                    }
                    Err(e) => {
                        eprintln!("Read error: {}", e);
                        break;
                    }
                }
            }

            Some(input) = input_rx.recv() => {
                if input == "/quit" {
                    println!("👋 Goodbye!");
                    break;
                }

                let msg = Message::PublicMessage {
                    username: username.clone(),
                    content: input,
                };

                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = write.write_all((json + "\n").as_bytes()).await;
                }
            }
        }
    }

    Ok(())
}

async fn login(write: &mut tokio::io::WriteHalf<tokio::net::TcpStream>) -> Result<String> {
    print!("Username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();

    print!("Password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim().to_string();

    let login_msg = Message::Login { username: username.clone(), password };
    let json = serde_json::to_string(&login_msg)? + "\n";
    write.write_all(json.as_bytes()).await?;

    Ok(username)
}