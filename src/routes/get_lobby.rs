use crate::core::app_state::AppState;
use crate::lobic_db::models::User;
use crate::schema::users::dsl::*;

use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::Response,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetLobbyResponse {
	pub id: String,
	pub lobby_name: String,
	pub lobby_icon: String,
	pub listeners: i32,
	pub song_name: String,
	pub artist_name: String,
}

pub async fn get_lobby(State(app_state): State<AppState>, Path(lobby_id): Path<String>) -> Response<String> {
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			let msg = format!("Failed to get DB from pool: {err}");
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(msg)
				.unwrap();
		}
	};

	// Getting the required lobby
	let lobby = match app_state.lobby_pool.get(&lobby_id) {
		Some(lobby) => lobby,
		None => {
			let msg = format!("Invalid lobby id: {}", lobby_id);
			return Response::builder().status(StatusCode::NOT_FOUND).body(msg).unwrap();
		}
	};

	// Getting the user data of the host
	let user = match users.filter(user_id.eq(&lobby.host_id)).first::<User>(&mut db_conn) {
		Ok(user) => user,
		Err(err) => {
			let msg = format!("Failed to fetch user: {err}");
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(msg)
				.unwrap();
		}
	};

	// Building the response
	let response = GetLobbyResponse {
		id: lobby_id,
		lobby_name: format!("{}'s Lobby", user.username),
		lobby_icon: lobby.music.image_url,
		listeners: lobby.clients.len() as i32,
		song_name: lobby.music.title,
		artist_name: lobby.music.artist,
	};

	let response_str = serde_json::to_string(&response).unwrap();
	Response::builder().status(StatusCode::OK).body(response_str).unwrap()
}
