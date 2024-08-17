use serde::{Deserialize, Serialize};
use uuid::Uuid;

// This struct represents a chat message
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub id: Uuid,
    pub user: String,
    pub content: String,
    pub channel: String,
    pub reply_to: Option<Uuid>, // ID of the message being replied to
    pub emotes: Vec<String>,    // Custom emotes
}

#[derive(Debug, Clone)]
pub enum ModerationAction {
    Kick(String),   // Kick a user from the channel
    Ban(String),    // Ban a user from the channel
    Unban(String),  // Unban a user
    Mute(String),   // Mute a user
}

impl ChatMessage {
    // Generate a new message with an ID
    pub fn new(user: String, content: String, channel: String, reply_to: Option<Uuid>, emotes: Vec<String>) -> Self {
        ChatMessage {
            id: Uuid::new_v4(),
            user,
            content,
            channel,
            reply_to,
            emotes,
        }
    }

    // Check if the message contains a moderation command
    pub fn parse_moderation_command(&self) -> Option<ModerationAction> {
        if self.content.starts_with("/kick ") {
            let user = self.content[6..].trim().to_string();
            Some(ModerationAction::Kick(user))
        } else if self.content.starts_with("/ban ") {
            let user = self.content[5..].trim().to_string();
            Some(ModerationAction::Ban(user))
        } else if self.content.starts_with("/unban ") {
            let user = self.content[7..].trim().to_string();
            Some(ModerationAction::Unban(user))
        } else if self.content.starts_with("/mute ") {
            let user = self.content[6..].trim().to_string();
            Some(ModerationAction::Mute(user))
        } else {
            None
        }
    }

    // Add functionality to validate message length
    pub fn is_valid(&self) -> bool {
        self.content.len() <= 500
    }
}