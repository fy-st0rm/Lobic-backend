use crate::core::app_state::AppState;
use crate::lobic_db::models::Playlist;
use axum::{extract::State, http::status::StatusCode, response::Response, Json};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewPlaylist {
	pub playlist_name: String,
	pub user_id: String,
	pub is_playlist_combined: bool,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse {
	pub message: String,
}

pub async fn create_playlist(State(app_state): State<AppState>, Json(payload): Json<NewPlaylist>) -> Response<String> {
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			let response = ApiResponse {
				message: format!("Failed to get DB from pool: {err}"),
			};
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(serde_json::to_string(&response).unwrap())
				.unwrap();
		}
	};

	use crate::schema::playlists::dsl::*;

	let curr_playlist_id = Uuid::new_v4(); //now a user can create a playlist with the same name

	let curr_creation_date_time = Utc::now().to_rfc3339();

	let new_playlist = Playlist {
		playlist_id: curr_playlist_id.to_string(),
		playlist_name: payload.playlist_name,
		user_id: payload.user_id,
		creation_date_time: curr_creation_date_time.clone(),
		last_updated_date_time: curr_creation_date_time,
		is_playlist_combined: payload.is_playlist_combined,
	};

	match diesel::insert_into(playlists)
		.values(&new_playlist)
		.execute(&mut db_conn)
	{
		Ok(_) => {
			let response = ApiResponse {
				message: format!("Playlist created with ID: {}", new_playlist.playlist_id),
			};
			Response::builder()
				.status(StatusCode::CREATED)
				.body(serde_json::to_string(&response).unwrap())
				.unwrap()
		}
		Err(err) => {
			let response = ApiResponse {
				message: format!("Failed to create playlist: {}", err),
			};
			Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(serde_json::to_string(&response).unwrap())
				.unwrap()
		}
	}
}
