use futures::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::accept_async;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Player {
    id: String,
    ready: bool,
    board: Option<Vec<Vec<u8>>>,
    piece: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Room {
    players: Vec<String>,
}

type Rooms = Arc<Mutex<HashMap<String, Room>>>;
type Players = Arc<Mutex<HashMap<String, Player>>>;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    CreateRoom { room_id: String },
    JoinRoom { room_id: String },
    LeaveRoom,
    Ready { ready: bool },
    BoardUpdate { board: Vec<Vec<u8>>, piece: String },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum ServerMessage {
    Connected { player_id: String },
    RoomCreated { room_id: String },
    JoinedRoom { room_id: String },
    PlayerJoined { player_id: String },
    PlayerLeft { player_id: String },
    PlayerReady { player_id: String, ready: bool },
    BoardUpdate { player_id: String, board: Vec<Vec<u8>>, piece: String },
}

async fn handle_connection(
    peer: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    rooms: Rooms,
    players: Players,
) {
    let mut ws = peer;
    let player_id = Uuid::new_v4().to_string();

    let player = Player {
        id: player_id.clone(),
        ready: false,
        board: None,
        piece: None,
    };
    players.lock().unwrap().insert(player_id.clone(), player);

    // Send connection message
    ws.send(Message::Text(
        serde_json::to_string(&ServerMessage::Connected { player_id: player_id.clone() }).unwrap(),
    ))
    .await
    .unwrap();

    while let Some(msg) = ws.next().await {
        if let Ok(msg) = msg {
            if msg.is_text() {
                let data = msg.to_text().unwrap();
                let client_msg: ClientMessage = serde_json::from_str(data).unwrap();
                let mut rooms_lock = rooms.lock().unwrap();
                let mut players_lock = players.lock().unwrap();

                match client_msg {
                    ClientMessage::CreateRoom { room_id } => {
                        if !rooms_lock.contains_key(&room_id) {
                            rooms_lock.insert(room_id.clone(), Room { players: vec![player_id.clone()] });
                            ws.send(Message::Text(serde_json::to_string(&ServerMessage::RoomCreated { room_id }).unwrap()))
                                .await.unwrap();
                        } else {
                            ws.send(Message::Text(r#"{"type": "error", "message": "Room already exists"}"#.to_string()))
                                .await.unwrap();
                        }
                    }
                    ClientMessage::JoinRoom { room_id } => {
                        if let Some(room) = rooms_lock.get_mut(&room_id) {
                            room.players.push(player_id.clone());
                            ws.send(Message::Text(serde_json::to_string(&ServerMessage::JoinedRoom { room_id: room_id.clone() }).unwrap()))
                                .await.unwrap();
                            let msg = serde_json::to_string(&ServerMessage::PlayerJoined { player_id: player_id.clone() }).unwrap();
                            for p_id in &room.players {
                                if let Some(player) = players_lock.get_mut(p_id) {
                                    ws.send(Message::Text(msg.clone())).await.unwrap();
                                }
                            }
                        } else {
                            ws.send(Message::Text(r#"{"type": "error", "message": "Room not found"}"#.to_string()))
                                .await.unwrap();
                        }
                    }
                    ClientMessage::LeaveRoom => {
                        if let Some(player) = players_lock.get(&player_id) {
                            if let Some(room) = player.room.clone() {
                                if let Some(room) = rooms_lock.get_mut(&room) {
                                    room.players.retain(|p_id| p_id != &player_id);
                                    let msg = serde_json::to_string(&ServerMessage::PlayerLeft { player_id: player_id.clone() }).unwrap();
                                    for p_id in &room.players {
                                        if let Some(player) = players_lock.get_mut(p_id) {
                                            ws.send(Message::Text(msg.clone())).await.unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ClientMessage::Ready { ready } => {
                        if let Some(player) = players_lock.get_mut(&player_id) {
                            player.ready = ready;
                            if let Some(room) = player.room.clone() {
                                if let Some(room) = rooms_lock.get(&room) {
                                    let msg = serde_json::to_string(&ServerMessage::PlayerReady {
                                        player_id: player_id.clone(),
                                        ready,
                                    })
                                    .unwrap();
                                    for p_id in &room.players {
                                        if let Some(player) = players_lock.get_mut(p_id) {
                                            ws.send(Message::Text(msg.clone())).await.unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ClientMessage::BoardUpdate { board, piece } => {
                        if let Some(player) = players_lock.get_mut(&player_id) {
                            player.board = Some(board.clone());
                            player.piece = Some(piece.clone());
                            if let Some(room) = player.room.clone() {
                                if let Some(room) = rooms_lock.get(&room) {
                                    let msg = serde_json::to_string(&ServerMessage::BoardUpdate {
                                        player_id: player_id.clone(),
                                        board,
                                        piece,
                                    })
                                    .unwrap();
                                    for p_id in &room.players {
                                        if let Some(player) = players_lock.get_mut(p_id) {
                                            ws.send(Message::Text(msg.clone())).await.unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.unwrap();

    let rooms = Arc::new(Mutex::new(HashMap::new()));
    let players = Arc::new(Mutex::new(HashMap::new()));

    println!("Tetris Multiplayer Server running on ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = accept_async(stream).await.unwrap();
        let rooms = Arc::clone(&rooms);
        let players = Arc::clone(&players);
        tokio::spawn(async move {
            handle_connection(peer, rooms, players).await;
        });
    }
}
