//! Shared library for **iron_server** — A modern Rust chat server.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{broadcast, RwLock};
use std::sync::Arc;

/// Default server address
pub const DEFAULT_ADDR: &str = "0.0.0.0:8080";

/// Broadcast channel capacity
pub const BROADCAST_CAPACITY: usize = 100;

/// Wire protocol — All messages between client and server
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    Login {
        username: String,
        password: String,
    },

    AuthSuccess {
        username: String,
    },

    AuthError {
        reason: String,
    },

    PublicMessage {
        username: String,
        content: String,
    },

    UserJoined {
        username: String,
    },

    UserLeft {
        username: String,
    },

    OnlineUsers {
        usernames: Vec<String>,
    },

    Shutdown,

    Error {
        reason: String,
    },
}

/// Online users store (thread-safe)
pub type OnlineUsers = Arc<RwLock<HashMap<String, ()>>>;

/// Broadcast sender
pub type BroadcastSender = broadcast::Sender<Message>;

/// Create new broadcast channel
pub fn new_broadcast() -> BroadcastSender {
    broadcast::channel(BROADCAST_CAPACITY).0
}

/// Demo users (username → password)
/// Replace with proper auth (JWT + DB) in production
pub fn demo_users() -> HashMap<String, String> {
    let mut users = HashMap::new();
    users.insert("alice".to_string(), "pass123".to_string());
    users.insert("bob".to_string(), "pass123".to_string());
    users.insert("carol".to_string(), "pass123".to_string());
    users
}