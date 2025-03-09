use crate::config::{MusicState, OpCode, SocketPayload, SocketResponse};
use crate::core::{
	app_state::AppState,
	lobby::{LobbyPool, Music},
	user_pool::UserPool,
};
use crate::lobic_db::db::*;
use crate::lobic_db::models::UserFriendship;
use crate::schema::user_friendship;

use axum::{
	extract::ws::{Message, WebSocket, WebSocketUpgrade},
	extract::State,
	response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::broadcast;
use diesel::prelude::*;

// :socket
pub async fn websocket_handler(ws: WebSocketUpgrade, State(app_state): State<AppState>) -> impl IntoResponse {
	ws.on_upgrade(|socket| handle_socket(socket, State(app_state)))
}

pub async fn handle_socket(socket: WebSocket, State(app_state): State<AppState>) {
	let (mut sender, mut receiver) = socket.split();
	let (tx, mut rx) = broadcast::channel(100);

	let db_pool = app_state.db_pool;
	let lobby_pool = app_state.lobby_pool;
	let user_pool = app_state.user_pool;

	// Receiving msg through sockets
	tokio::spawn(async move {
		// Temporary user state
		let mut user_id: Option<String> = None;
		let mut curr_lobby_id: Option<String> = None;

		while let Some(Ok(message)) = receiver.next().await {
			if let Message::Text(text) = message {
				// Extracting payload
				let payload: SocketPayload = match serde_json::from_str(&text) {
					Ok(value) => value,
					Err(err) => {
						let response = json!({
							"op_code": OpCode::ERROR,
							"value": err.to_string()
						})
						.to_string();
						let _ = tx.send(Message::Text(response));
						return;
					}
				};

				// Operating according to the opcode
				let response = match payload.op_code {
					OpCode::CONNECT => handle_connect(&tx, payload.value, &db_pool, &user_pool),
					OpCode::CREATE_LOBBY => handle_create_lobby(payload.value, &db_pool, &lobby_pool, &user_pool),
					OpCode::JOIN_LOBBY => handle_join_lobby(payload.value, &db_pool, &lobby_pool, &user_pool),
					OpCode::LEAVE_LOBBY => handle_leave_lobby(payload.value, &db_pool, &lobby_pool, &user_pool),
					OpCode::GET_LOBBY_IDS => handle_get_lobby_ids(payload.value, &db_pool, &lobby_pool),
					OpCode::GET_LOBBY_MEMBERS => handle_get_lobby_members(payload.value, &lobby_pool),
					OpCode::MESSAGE => handle_message(payload.value, &db_pool, &lobby_pool, &user_pool),
					OpCode::GET_MESSAGES => handle_get_messages(payload.value, &lobby_pool),
					OpCode::SET_MUSIC_STATE => handle_set_music_state(payload.value, &lobby_pool, &user_pool),
					OpCode::SYNC_MUSIC => handle_sync_music(payload.value, &lobby_pool),
					OpCode::SET_QUEUE => handle_set_queue(payload.value, &lobby_pool, &user_pool),
					OpCode::SYNC_QUEUE => handle_sync_queue(payload.value, &lobby_pool),
					OpCode::REQUEST_MUSIC_PLAY => {
						handle_request_music_play(payload.value, &lobby_pool, &user_pool, &db_pool)
					}
					_ => Err(format!("Invalid opcode: {:?}", payload.op_code)),
				};

				// Returning response to the client
				match response {
					Ok(soc_res) => {
						// Storing some intermidiate data
						match soc_res.r#for {
							OpCode::CONNECT => {
								user_id = Some(soc_res.value.as_str().unwrap().to_string());
							}
							OpCode::CREATE_LOBBY => {
								curr_lobby_id =
									Some(soc_res.value.get("lobby_id").unwrap().as_str().unwrap().to_string());
							}
							OpCode::JOIN_LOBBY => {
								curr_lobby_id =
									Some(soc_res.value.get("lobby_id").unwrap().as_str().unwrap().to_string());
							}
							OpCode::LEAVE_LOBBY => {
								curr_lobby_id = None;
							}
							_ => (),
						};

						// Sending the final response
						let _ = tx.send(Message::Text(soc_res.to_string()));
					}
					Err(err) => {
						let msg = json!({
							"op_code": OpCode::ERROR,
							"value": err
						})
						.to_string();
						let _ = tx.send(Message::Text(msg));
					}
				}
			}
		}

		// If the user suddenly disconnects, disconnect the user from the lobby
		if let Some(lobby_id) = curr_lobby_id {
			let payload = json!({
				"lobby_id": lobby_id,
				"user_id": user_id.unwrap(),
			});
			let _ = handle_leave_lobby(payload, &db_pool, &lobby_pool, &user_pool);
		}
	});

	// Sending msg through sockets
	tokio::spawn(async move {
		while let Ok(msg) = rx.recv().await {
			if sender.send(msg).await.is_err() {
				break;
			}
		}
	});
}

// Endpoint handlers

// :connect
#[derive(Serialize, Deserialize)]
struct ConnectPayload {
	pub user_id: String,
}

fn handle_connect(
	tx: &broadcast::Sender<Message>,
	value: Value,
	db_pool: &DatabasePool,
	user_pool: &UserPool,
) -> Result<SocketResponse, String> {
	let payload: ConnectPayload = serde_json::from_value(value).map_err(|x| x.to_string())?;

	if !user_exists(&payload.user_id, db_pool) {
		return Err(format!("Invalid user_id: {}", payload.user_id));
	}

	let response = SocketResponse {
		op_code: OpCode::OK,
		r#for: OpCode::CONNECT,
		value: payload.user_id.clone().into(),
	};

	user_pool.insert(&payload.user_id, tx);

	Ok(response)
}

// :create_lobby
#[derive(Serialize, Deserialize)]
struct CreateLobbyPayload {
	pub host_id: String,
}

fn handle_create_lobby(
	value: Value,
	db_pool: &DatabasePool,
	lobby_pool: &LobbyPool,
	user_pool: &UserPool,
) -> Result<SocketResponse, String> {
	let payload: CreateLobbyPayload = serde_json::from_value(value).map_err(|x| x.to_string())?;

	let res = lobby_pool.create_lobby(&payload.host_id, db_pool)?;

	let response = SocketResponse {
		op_code: OpCode::OK,
		r#for: OpCode::CREATE_LOBBY,
		value: res,
	};

	// Getting db connection
	let mut db_conn = db_pool.get().unwrap();

	// Loading the friendship of the host
	let friendships = user_friendship::table
		.filter(user_friendship::user_id.eq(&payload.host_id))
		.load::<UserFriendship>(&mut db_conn)
		.unwrap();

	// Collecting all the friends ids
	let friends: Vec<String> = friendships
		.iter()
		.map(|f| f.friend_id.clone())
		.collect();

	// Broadcasting to friends
	let user_ids = user_pool.get_ids();
	for user_id in user_ids {
		if friends.contains(&user_id) {
			let conn = user_pool.get(&user_id).unwrap();
			let ids = lobby_pool.get_ids_with_rel(user_id.clone(), db_pool);
			let response = SocketResponse {
				op_code: OpCode::OK,
				r#for: OpCode::GET_LOBBY_IDS,
				value: ids.into(),
			}
			.to_string();
			let _ = conn.send(Message::Text(response));
		}
	}

	Ok(response)
}

// :join_lobby
#[derive(Serialize, Deserialize)]
struct JoinLobbyPayload {
	pub lobby_id: String,
	pub user_id: String,
}

fn handle_join_lobby(
	value: Value,
	db_pool: &DatabasePool,
	lobby_pool: &LobbyPool,
	user_pool: &UserPool,
) -> Result<SocketResponse, String> {
	let payload: JoinLobbyPayload = serde_json::from_value(value).map_err(|x| x.to_string())?;

	let res = lobby_pool.join_lobby(&payload.lobby_id, &payload.user_id, db_pool, user_pool)?;
	let response = SocketResponse {
		op_code: OpCode::OK,
		r#for: OpCode::JOIN_LOBBY,
		value: res.into(),
	};

	Ok(response)
}

// :leave_lobby
#[derive(Debug, Serialize, Deserialize)]
struct LeaveLobbyPayload {
	pub lobby_id: String,
	pub user_id: String,
}

fn handle_leave_lobby(
	value: Value,
	db_pool: &DatabasePool,
	lobby_pool: &LobbyPool,
	user_pool: &UserPool,
) -> Result<SocketResponse, String> {
	let payload: LeaveLobbyPayload = serde_json::from_value(value).map_err(|x| x.to_string())?;

	let lobby = match lobby_pool.get(&payload.lobby_id) {
		Some(lobby) => lobby,
		None => return Err(format!("Invalid lobby id: {}", payload.lobby_id)),
	};

	// If the user is host of the lobby, the lobby gets deleted when host leaves.
	let res: Result<String, String>;
	if lobby.host_id == payload.user_id {
		res = lobby_pool.delete_lobby(&payload.lobby_id, user_pool);

		// Getting db connection
		let mut db_conn = db_pool.get().unwrap();

		// Loading the friendship of the host
		let friendships = user_friendship::table
			.filter(user_friendship::user_id.eq(&payload.user_id))
			.load::<UserFriendship>(&mut db_conn)
			.unwrap();

		// Collecting all the friends ids
		let friends: Vec<String> = friendships
			.iter()
			.map(|f| f.friend_id.clone())
			.collect();

		// Broadcasting to friends
		let user_ids = user_pool.get_ids();
		for user_id in user_ids {
			if friends.contains(&user_id) {
				let conn = user_pool.get(&user_id).unwrap();
				let ids = lobby_pool.get_ids_with_rel(user_id.clone(), db_pool);
				let response = SocketResponse {
					op_code: OpCode::OK,
					r#for: OpCode::GET_LOBBY_IDS,
					value: ids.into(),
				}
				.to_string();
				let _ = conn.send(Message::Text(response));
			}
		}
	} else {
		res = lobby_pool.leave_lobby(&payload.lobby_id, &payload.user_id, db_pool, user_pool);
	}

	let ok = res?;
	let response = SocketResponse {
		op_code: OpCode::OK,
		r#for: OpCode::LEAVE_LOBBY,
		value: ok.into(),
	};

	Ok(response)
}

// :get_lobby_ids
#[derive(Serialize, Deserialize)]
struct GetLobbyIdsPayload {
	pub user_id: String,
}

fn handle_get_lobby_ids(value: Value, db_pool: &DatabasePool, lobby_pool: &LobbyPool) -> Result<SocketResponse, String> {
	let payload: GetLobbyIdsPayload = serde_json::from_value(value).map_err(|x| x.to_string())?;
	let ids = lobby_pool.get_ids_with_rel(payload.user_id, db_pool);
	let response = SocketResponse {
		op_code: OpCode::OK,
		r#for: OpCode::GET_LOBBY_IDS,
		value: ids.into(),
	};
	Ok(response)
}

// :get_lobby_members
#[derive(Serialize, Deserialize)]
struct GetLobbyMembersPayload {
	pub lobby_id: String,
}

fn handle_get_lobby_members(value: Value, lobby_pool: &LobbyPool) -> Result<SocketResponse, String> {
	let payload: GetLobbyMembersPayload = serde_json::from_value(value).map_err(|x| x.to_string())?;

	let lobby = match lobby_pool.get(&payload.lobby_id) {
		Some(lobby) => lobby,
		None => return Err(format!("Invalid lobby id: {}", payload.lobby_id)),
	};

	let clients = lobby.clients;
	let response = SocketResponse {
		op_code: OpCode::OK,
		r#for: OpCode::GET_LOBBY_MEMBERS,
		value: clients.into(),
	};

	Ok(response)
}

// :message
#[derive(Serialize, Deserialize)]
struct MessagePayload {
	pub lobby_id: String,
	pub user_id: String,
	pub message: String,
}

fn handle_message(
	value: Value,
	db_pool: &DatabasePool,
	lobby_pool: &LobbyPool,
	user_pool: &UserPool,
) -> Result<SocketResponse, String> {
	let payload: MessagePayload = serde_json::from_value(value).map_err(|x| x.to_string())?;

	lobby_pool.append_message(&payload.lobby_id, &payload.user_id, &payload.message, db_pool)?;

	let lobby = lobby_pool.get(&payload.lobby_id).unwrap(); // unwrapped cuz we're sure the lobby exists cuz of above function call. i hope..
	let msgs = lobby.chat;

	// Broadcasting the message to everyone in the lobby
	for client_id in lobby.clients {
		let response = SocketResponse {
			op_code: OpCode::OK,
			r#for: OpCode::GET_MESSAGES,
			value: msgs.clone().into(),
		}
		.to_string();

		let client_conn = match user_pool.get(&client_id) {
			Some(conn) => conn,
			None => {
				return Err(format!(
					"Cannot find user {} in a lobby {} (in \"handle_message\" this shouldnt occure)",
					client_id, payload.lobby_id
				));
			}
		};
		let _ = client_conn.send(Message::Text(response));
	}

	let response = SocketResponse {
		op_code: OpCode::OK,
		r#for: OpCode::MESSAGE,
		value: "Sucessfully sent message".into(),
	};

	Ok(response)
}

// :get_message
#[derive(Serialize, Deserialize)]
struct GetMessagePayload {
	pub lobby_id: String,
}

fn handle_get_messages(value: Value, lobby_pool: &LobbyPool) -> Result<SocketResponse, String> {
	let payload: GetMessagePayload = serde_json::from_value(value).map_err(|x| x.to_string())?;

	let msgs = match lobby_pool.get_msgs(&payload.lobby_id) {
		Some(msgs) => msgs,
		None => return Err(format!("Invalid lobby id: {}", payload.lobby_id)),
	};

	let response = SocketResponse {
		op_code: OpCode::OK,
		r#for: OpCode::GET_MESSAGES,
		value: msgs.into(),
	};

	Ok(response)
}

// :set_music_state
#[derive(Serialize, Deserialize)]
struct SetMusicStatePayload {
	pub lobby_id: String,
	pub user_id: String,
	pub music_id: String,
	pub title: String,
	pub artist: String,
	pub image_url: String,
	pub timestamp: f64,
	pub state: MusicState,
}

fn handle_set_music_state(
	value: Value,
	lobby_pool: &LobbyPool,
	user_pool: &UserPool,
) -> Result<SocketResponse, String> {
	let payload: SetMusicStatePayload = serde_json::from_value(value).map_err(|x| x.to_string())?;

	let music = Music {
		id: payload.music_id,
		title: payload.title,
		artist: payload.artist,
		image_url: payload.image_url,
		timestamp: payload.timestamp,
		state: payload.state,
	};

	lobby_pool.set_music_state(&payload.lobby_id, &payload.user_id, music)?;

	let lobby = lobby_pool.get(&payload.lobby_id).unwrap();
	let music = lobby.music;

	// Sending the sync request to every client in lobby
	for client_id in lobby.clients {
		if client_id == payload.user_id {
			continue;
		}

		let response = SocketResponse {
			op_code: OpCode::OK,
			r#for: OpCode::SYNC_MUSIC,
			value: music.clone().into(),
		}
		.to_string();

		let client_conn = match user_pool.get(&client_id) {
			Some(conn) => conn,
			None => {
				return Err(format!(
					"Cannot find user {} in a lobby {} (in \"handle_set_music_state\" this shouldnt occure)",
					client_id, payload.lobby_id
				));
			}
		};
		let _ = client_conn.send(Message::Text(response));
	}

	let response = SocketResponse {
		op_code: OpCode::OK,
		r#for: OpCode::SET_MUSIC_STATE,
		value: "Sucessfully set music state".into(),
	};

	Ok(response)
}

// :sync_music
#[derive(Serialize, Deserialize)]
struct SyncMusicPayload {
	pub lobby_id: String,
	pub current_state: MusicState,
}

fn handle_sync_music(value: Value, lobby_pool: &LobbyPool) -> Result<SocketResponse, String> {
	let payload: SyncMusicPayload = serde_json::from_value(value).map_err(|x| x.to_string())?;

	let lobby = match lobby_pool.get(&payload.lobby_id) {
		Some(lobby) => lobby,
		None => return Err(format!("Invalid lobby id: {}", payload.lobby_id)),
	};

	let mut music = lobby.music;

	if payload.current_state == MusicState::EMPTY && music.id.len() > 0 {
		music.state = MusicState::CHANGE_MUSIC;
	}

	let response = SocketResponse {
		op_code: OpCode::SYNC_MUSIC,
		r#for: OpCode::SYNC_MUSIC,
		value: music.into(),
	};

	Ok(response)
}

// :set_queue
#[derive(Serialize, Deserialize)]
struct SetQueuePayload {
	pub lobby_id: String,
	pub queue: Vec<Music>,
}

fn handle_set_queue(value: Value, lobby_pool: &LobbyPool, user_pool: &UserPool) -> Result<SocketResponse, String> {
	let payload: SetQueuePayload = serde_json::from_value(value).map_err(|x| x.to_string())?;

	lobby_pool.set_queue(&payload.lobby_id, payload.queue)?;

	let lobby = lobby_pool.get(&payload.lobby_id).unwrap();
	let queue = lobby.queue;

	// Sending the sync request to every client in lobby
	for client_id in lobby.clients {
		if client_id == lobby.host_id {
			continue;
		}

		let response = SocketResponse {
			op_code: OpCode::OK,
			r#for: OpCode::SYNC_QUEUE,
			value: queue.clone().into(),
		}
		.to_string();

		let client_conn = match user_pool.get(&client_id) {
			Some(conn) => conn,
			None => {
				return Err(format!(
					"Cannot find user {} in a lobby {} (in \"handle_set_music_state\" this shouldnt occure)",
					client_id, payload.lobby_id
				));
			}
		};
		let _ = client_conn.send(Message::Text(response));
	}

	let response = SocketResponse {
		op_code: OpCode::OK,
		r#for: OpCode::SET_QUEUE,
		value: "Sucessfully set queue".into(),
	};

	Ok(response)
}

// :sync_queue
#[derive(Serialize, Deserialize)]
struct SyncQueuePayload {
	pub lobby_id: String,
}

fn handle_sync_queue(value: Value, lobby_pool: &LobbyPool) -> Result<SocketResponse, String> {
	let payload: SyncQueuePayload = serde_json::from_value(value).map_err(|x| x.to_string())?;

	let lobby = match lobby_pool.get(&payload.lobby_id) {
		Some(lobby) => lobby,
		None => return Err(format!("Invalid lobby id: {}", payload.lobby_id)),
	};

	let response = SocketResponse {
		op_code: OpCode::OK,
		r#for: OpCode::SYNC_MUSIC,
		value: lobby.queue.into(),
	};

	Ok(response)
}

// :request_music_play
#[derive(Serialize, Deserialize)]
struct RequestMusicPlayPayload {
	pub lobby_id: String,
	pub music: Music,
}

fn handle_request_music_play(
	value: Value,
	lobby_pool: &LobbyPool,
	user_pool: &UserPool,
	db_pool: &DatabasePool,
) -> Result<SocketResponse, String> {
	let payload: RequestMusicPlayPayload = serde_json::from_value(value).map_err(|x| x.to_string())?;
	lobby_pool.add_requested_music(&payload.lobby_id, payload.music, user_pool, db_pool)?;

	let response = SocketResponse {
		op_code: OpCode::OK,
		r#for: OpCode::REQUEST_MUSIC_PLAY,
		value: "Sucessfully requested the music".into(),
	};

	Ok(response)
}
