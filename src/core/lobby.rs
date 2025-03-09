use crate::config::{MusicState, OpCode, SocketResponse};
use crate::core::user_pool::UserPool;
use crate::lobic_db::db::*;
use crate::lobic_db::models::Notification;
use crate::routes::notify::notify;
use crate::utils::timestamp;
use crate::lobic_db::models::UserFriendship;
use crate::schema::user_friendship;

use diesel::prelude::*;
use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatValue {
	pub user_id: String,
	pub message: String,
	pub timestamp: String,
}
type Chat = Vec<ChatValue>;

impl From<ChatValue> for Value {
	fn from(chat_value: ChatValue) -> Self {
		serde_json::to_value(&chat_value).unwrap()
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Music {
	pub id: String,
	pub title: String,
	pub artist: String,
	pub image_url: String,
	pub timestamp: f64,
	pub state: MusicState,
}

impl Music {
	pub fn new() -> Music {
		Music {
			id: String::new(),
			title: String::new(),
			artist: String::new(),
			image_url: String::new(),
			timestamp: 0.0,
			state: MusicState::PAUSE,
		}
	}
}

impl From<Music> for Value {
	fn from(music: Music) -> Self {
		serde_json::to_value(&music).unwrap()
	}
}

#[derive(Debug, Clone)]
pub struct Lobby {
	pub id: String,
	pub host_id: String,
	pub clients: Vec<String>,
	pub chat: Chat,
	pub music: Music,
	pub queue: Vec<Music>,
	pub requested_musics: HashMap<String, Music>,
}

#[derive(Debug, Clone)]
pub struct LobbyPool {
	inner: Arc<Mutex<HashMap<String, Lobby>>>,
}

impl LobbyPool {
	pub fn new() -> LobbyPool {
		LobbyPool {
			inner: Arc::new(Mutex::new(HashMap::new())),
		}
	}

	pub fn exists(&self, key: &str) -> bool {
		let inner = self.inner.lock().unwrap();
		inner.contains_key(key)
	}

	pub fn get_ids(&self) -> Vec<String> {
		let inner = self.inner.lock().unwrap();
		inner.clone().into_keys().collect()
	}

	// Retrives the ids of lobby in which host is there friend
	pub fn get_ids_with_rel(&self, user_id: String, db_pool: &DatabasePool) -> Vec<String> {
		let mut db_conn = db_pool.get().unwrap();

		let mut lobby_ids: Vec<String> = Vec::new();

		let inner = self.inner.lock().unwrap();
		for (lobby_id, lobby) in inner.clone().into_iter() {
			let host_id = lobby.host_id;

			// Loading the friendship of the host
			let friendships = user_friendship::table
				.filter(user_friendship::user_id.eq(&host_id))
				.load::<UserFriendship>(&mut db_conn)
				.unwrap();

			// Collecting all the friends ids
			let friends: Vec<String> = friendships
				.iter()
				.map(|f| f.friend_id.clone())
				.collect();

			if friends.contains(&user_id) {
				lobby_ids.push(lobby_id);
			}
		}

		return lobby_ids;
	}

	pub fn get(&self, key: &str) -> Option<Lobby> {
		let inner = self.inner.lock().unwrap();
		inner.get(key).cloned()
	}

	pub fn get_msgs(&self, lobby_id: &str) -> Option<Chat> {
		if !self.exists(lobby_id) {
			return None;
		}
		let lobby = self.get(lobby_id).unwrap();
		Some(lobby.chat)
	}

	pub fn insert(&self, key: &str, lobby: Lobby) {
		let mut inner = self.inner.lock().unwrap();
		inner.insert(key.to_string(), lobby);
	}

	pub fn create_lobby(&self, host_id: &str, db_pool: &DatabasePool) -> Result<Value, String> {
		if !user_exists(host_id, db_pool) {
			return Err(format!("Invalid host id: {}", host_id));
		}

		// Generating lobby id
		let mut lobby_id = Uuid::new_v4().to_string();
		while self.exists(&lobby_id) {
			lobby_id = Uuid::new_v4().to_string();
		}

		// Constructing lobby
		let lobby = Lobby {
			id: lobby_id.clone(),
			host_id: host_id.to_string(),
			clients: vec![host_id.to_string()],
			chat: Vec::new(),
			music: Music::new(),
			queue: Vec::new(),
			requested_musics: HashMap::new(),
		};
		self.insert(&lobby_id, lobby);

		// Constructing response
		let response = json!({
			"lobby_id": lobby_id
		});

		Ok(response)
	}

	pub fn join_lobby(
		&self,
		lobby_id: &str,
		client_id: &str,
		db_pool: &DatabasePool,
		user_pool: &UserPool,
	) -> Result<Value, String> {
		if !user_exists(client_id, db_pool) {
			return Err(format!("Invalid client id: {}", client_id));
		}

		// Getting the lobby to be joined
		let mut lobby = match self.get(lobby_id) {
			Some(lobby) => lobby,
			None => {
				return Err(format!("Invalid lobby id: {}", lobby_id));
			}
		};

		// Check if client is already joined
		if lobby.clients.contains(&client_id.to_string()) {
			return Err(format!("Client: {} is already in lobby: {}", client_id, lobby_id));
		}

		// Adding the client
		lobby.clients.push(client_id.to_string());

		// Broadcasting to the members of the lobby that someone has left
		for client in &lobby.clients {
			if let Some(conn) = user_pool.get(&client) {
				let response = SocketResponse {
					op_code: OpCode::OK,
					r#for: OpCode::GET_LOBBY_MEMBERS,
					value: lobby.clients.clone().into(),
				}
				.to_string();
				let _ = conn.send(Message::Text(response));
			}
		}

		// Pushing the new lobby
		self.insert(lobby_id, lobby);

		// Constructing response
		let response = json!({
			"lobby_id": lobby_id
		});

		Ok(response)
	}

	pub fn leave_lobby(
		&self,
		lobby_id: &str,
		client_id: &str,
		db_pool: &DatabasePool,
		user_pool: &UserPool,
	) -> Result<String, String> {
		if !user_exists(client_id, db_pool) {
			return Err(format!("Invalid client id: {}", client_id));
		}

		// Getting the lobby to leave
		let mut inner = self.inner.lock().unwrap();
		let lobby = match inner.get_mut(lobby_id) {
			Some(lobby) => lobby,
			None => {
				return Err(format!("Invalid lobby id: {}", lobby_id));
			}
		};

		lobby.clients.retain(|id| id != client_id);

		// Broadcasting to the members of the lobby that someone has left
		for client in &lobby.clients {
			if let Some(conn) = user_pool.get(&client) {
				let response = SocketResponse {
					op_code: OpCode::OK,
					r#for: OpCode::GET_LOBBY_MEMBERS,
					value: lobby.clients.clone().into(),
				}
				.to_string();
				let _ = conn.send(Message::Text(response));
			}
		}

		Ok("Sucessfully left lobby".to_string())
	}

	pub fn delete_lobby(&self, lobby_id: &str, user_pool: &UserPool) -> Result<String, String> {
		// Getting the lobby to leave
		let lobby = match self.get(lobby_id) {
			Some(lobby) => lobby,
			None => {
				return Err(format!("Invalid lobby id: {}", lobby_id));
			}
		};

		// Notifying all the clients in the lobby to leave
		for client in lobby.clients {
			if let Some(conn) = user_pool.get(&client) {
				let response = SocketResponse {
					op_code: OpCode::OK,
					r#for: OpCode::LEAVE_LOBBY,
					value: "Host disconnected".into(),
				}
				.to_string();
				let _ = conn.send(Message::Text(response));
			}
		}

		// Deleting the lobby
		let mut inner = self.inner.lock().unwrap();
		let _ = inner.remove(lobby_id);

		Ok("Sucessfully deleted lobby".to_string())
	}

	pub fn append_message(
		&self,
		lobby_id: &str,
		client_id: &str,
		msg: &str,
		db_pool: &DatabasePool,
	) -> Result<(), String> {
		if !user_exists(client_id, db_pool) {
			return Err(format!("Invalid client id: {}", client_id));
		}

		let mut inner = self.inner.lock().unwrap();
		let lobby = match inner.get_mut(lobby_id) {
			Some(lobby) => lobby,
			None => {
				return Err(format!("Invalid lobby id: {}", lobby_id));
			}
		};

		if !lobby.clients.contains(&client_id.to_string()) {
			return Err(format!("Client: {} is not a member in lobby: {}", client_id, lobby_id));
		}

		lobby.chat.push(ChatValue {
			user_id: client_id.to_string(),
			message: msg.to_string(),
			timestamp: timestamp::now(),
		});
		Ok(())
	}

	pub fn set_music_state(&self, lobby_id: &str, user_id: &str, music: Music) -> Result<(), String> {
		let mut inner = self.inner.lock().unwrap();
		let lobby = match inner.get_mut(lobby_id) {
			Some(lobby) => lobby,
			None => {
				return Err(format!("Invalid lobby id: {}", lobby_id));
			}
		};

		if lobby.host_id != user_id {
			return Err(format!("User {} is not the host of lobby {}", user_id, lobby_id));
		}
		lobby.music = music;
		Ok(())
	}

	pub fn set_queue(&self, lobby_id: &str, queue: Vec<Music>) -> Result<(), String> {
		let mut inner = self.inner.lock().unwrap();
		let lobby = match inner.get_mut(lobby_id) {
			Some(lobby) => lobby,
			None => {
				return Err(format!("Invalid lobby id: {}", lobby_id));
			}
		};

		lobby.queue = queue;
		Ok(())
	}

	pub fn add_requested_music(
		&self,
		lobby_id: &str,
		music: Music,
		user_pool: &UserPool,
		db_pool: &DatabasePool,
	) -> Result<(), String> {
		let mut inner = self.inner.lock().unwrap();
		let lobby = match inner.get_mut(lobby_id) {
			Some(lobby) => lobby,
			None => {
				return Err(format!("Invalid lobby id: {}", lobby_id));
			}
		};

		// Send the host a notification for this
		let notif = Notification::new(OpCode::REQUEST_MUSIC_PLAY, music.clone().into());
		notify(&lobby.host_id, notif, db_pool, user_pool);

		lobby.requested_musics.insert(music.id.clone(), music);
		Ok(())
	}
}
