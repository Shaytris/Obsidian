use crate::message::ChatMessage;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use futures::stream::SplitSink;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;

pub struct ChannelManager {
    pub channels: HashMap<String, Vec<ChatMessage>>,
    pub users: HashMap<String, HashSet<UserConnection>>,
    pub banned_users: HashMap<String, HashSet<String>>, // Banned users per channel
    pub muted_users: HashMap<String, HashSet<String>>,  // Muted users per channel
}

pub struct UserConnection {
    pub username: String,
    pub write: SplitSink<WebSocketStream<TcpStream>, Message>,
}

impl ChannelManager {
    pub fn new() -> Self {
        ChannelManager {
            channels: HashMap::new(),
            users: HashMap::new(),
            banned_users: HashMap::new(),
            muted_users: HashMap::new(),
        }
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        let channel_messages = self.channels.entry(message.channel.clone()).or_insert(Vec::new());
        channel_messages.push(message);
    }

    pub fn get_channel_messages(&self, channel: &str) -> Option<&Vec<ChatMessage>> {
        self.channels.get(channel)
    }

    pub fn add_user(&mut self, channel: &str, user: UserConnection) {
        let users = self.users.entry(channel.to_string()).or_insert(HashSet::new());
        users.insert(user);
    }

    pub fn kick_user(&mut self, channel: &str, user: &str) {
        if let Some(users) = self.users.get_mut(channel) {
            users.retain(|u| u.username != user);
        }
    }

    pub fn ban_user(&mut self, channel: &str, user: &str) {
        let bans = self.banned_users.entry(channel.to_string()).or_insert(HashSet::new());
        bans.insert(user.to_string());
        self.kick_user(channel, user);
    }

    pub fn unban_user(&mut self, channel: &str, user: &str) {
        if let Some(bans) = self.banned_users.get_mut(channel) {
            bans.remove(user);
        }
    }

    pub fn mute_user(&mut self, channel: &str, user: &str) {
        let mutes = self.muted_users.entry(channel.to_string()).or_insert(HashSet::new());
        mutes.insert(user.to_string());
    }

    pub fn unmute_user(&mut self, channel: &str, user: &str) {
        if let Some(mutes) = self.muted_users.get_mut(channel) {
            mutes.remove(user);
        }
    }

    pub fn is_muted(&self, channel: &str, user: &str) -> bool {
        if let Some(mutes) = self.muted_users.get(channel) {
            return mutes.contains(user);
        }
        false
    }

    pub fn get_connected_users(&self, channel: &str) -> Option<&HashSet<UserConnection>> {
        self.users.get(channel)
    }
}

impl PartialEq for UserConnection {
    fn eq(&self, other: &Self) -> bool {
        self.username == other.username
    }
}

impl Eq for UserConnection {}

impl std::hash::Hash for UserConnection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.username.hash(state);
    }
}