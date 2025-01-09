use crate::core::app_state::AppState;
use crate::lobic_db::models::Playlist;
use axum::{extract::State, http::status::StatusCode, response::Response, Json};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewPlaylist {
	pub playlist_name: String,
	pub user_id: String,
	pub description: Option<String>,
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

	// Generate playlist_id using DefaultHasher
	let mut hasher = DefaultHasher::new();
	payload.user_id.hash(&mut hasher);
	payload.playlist_name.hash(&mut hasher);
	let hash = hasher.finish();

	// Convert the hash to a UUID
	let curr_playlist_id = Uuid::from_u64_pair(hash, hash).to_string();

	// Check if a playlist with the same user_id and playlist_name already exists
	let existing_playlist = playlists
		.filter(user_id.eq(&payload.user_id))
		.filter(playlist_name.eq(&payload.playlist_name))
		.first::<Playlist>(&mut db_conn);

	match existing_playlist {
		Ok(_) => {
			let response = ApiResponse {
				message: "A playlist with the same name already exists for this user".to_string(),
			};
			return Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(serde_json::to_string(&response).unwrap())
				.unwrap();
		}
		Err(diesel::result::Error::NotFound) => {
			// No existing playlist found, proceed to create a new one
		}
		Err(err) => {
			let response = ApiResponse {
				message: format!("Failed to query database: {}", err),
			};
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(serde_json::to_string(&response).unwrap())
				.unwrap();
		}
	}

	let curr_creation_date_time = Utc::now().to_rfc3339();

	let new_playlist = Playlist {
		playlist_id: curr_playlist_id.clone(),
		playlist_name: payload.playlist_name,
		user_id: payload.user_id,
		description: payload.description,
		creation_date_time: curr_creation_date_time.clone(),
		last_updated_date_time: curr_creation_date_time,
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
