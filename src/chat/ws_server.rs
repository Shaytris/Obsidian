use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio::net::TcpStream;
use futures::{StreamExt, SinkExt};
use serde_json::json;
use crate::message::{ChatMessage, ModerationAction};
use crate::channel::{ChannelManager, handle_message};
use crate::utils::{process_emotes, save_message_to_db};

// Handle the incoming WebSocket connection
pub async fn handle_connection(stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();

    let mut channel_manager = ChannelManager::new();

    while let Some(Ok(message)) = read.next().await {
        if let Ok(text) = message.to_text() {
            let incoming_message: ChatMessage = serde_json::from_str(text)?;
            
            // Handle moderation actions if present
            if let Some(action) = incoming_message.parse_moderation_command() {
                handle_moderation(action, &mut channel_manager, incoming_message.channel.clone());
                continue;
            }

            // Handle incoming message and process custom emotes
            let processed_message = handle_message(incoming_message.clone(), &mut channel_manager);
            let processed_content = process_emotes(processed_message.content.clone(), &processed_message.emotes);

            // Save message to the database
            save_message_to_db(&processed_message).await?;

            // Broadcast the message to all connected users in the channel
            let broadcast_message = json!({
                "id": processed_message.id,
                "user": processed_message.user,
                "content": processed_content,
                "channel": processed_message.channel,
                "reply_to": processed_message.reply_to
            });

            broadcast_to_channel(&mut channel_manager, processed_message.channel.clone(), broadcast_message.to_string()).await?;
        }
    }

    Ok(())
}

async fn broadcast_to_channel(manager: &mut ChannelManager, channel: String, message: String) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(users) = manager.get_connected_users(&channel) {
        for user in users {
            user.write.send(Message::text(message.clone())).await?;
        }
    }
    Ok(())
}

// Handle moderation actions
fn handle_moderation(action: ModerationAction, manager: &mut ChannelManager, channel: String) {
    match action {
        ModerationAction::Kick(user) => {
            manager.kick_user(&channel, &user);
            println!("Kicked user {} from channel {}", user, channel);
        }
        ModerationAction::Ban(user) => {
            manager.ban_user(&channel, &user);
            println!("Banned user {} from channel {}", user, channel);
        }
        ModerationAction::Unban(user) => {
            manager.unban_user(&channel, &user);
            println!("Unbanned user {} from channel {}", user, channel);
        }
        ModerationAction::Mute(user) => {
            manager.mute_user(&channel, &user);
            println!("Muted user {} in channel {}", user, channel);
        }
    }
}