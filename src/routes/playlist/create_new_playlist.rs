use std::fs;
use std::path::Path;

use crate::lobic_db::models::Playlist;
use crate::{config::PLAYLIST_COVER_IMG_STORAGE, core::app_state::AppState};
use axum::{
	body::Bytes,
	extract::{Query, State},
	http::status::StatusCode,
	response::Response,
};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistParams {
	pub playlist_name: String,
	pub user_id: String,
	pub is_playlist_combined: bool,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse {
	pub message: String,
}

pub async fn create_playlist(
	State(app_state): State<AppState>,
	Query(params): Query<PlaylistParams>,
	body: Bytes,
) -> Response<String> {
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
		playlist_name: params.playlist_name,
		user_id: params.user_id,
		creation_date_time: curr_creation_date_time.clone(),
		last_updated_date_time: curr_creation_date_time,
		is_playlist_combined: params.is_playlist_combined,
	};

	//save the image inside the storage
	let storage_path = Path::new(PLAYLIST_COVER_IMG_STORAGE);
	if let Err(err) = fs::create_dir_all(storage_path) {
		return Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Failed to create directory: {}", err))
			.unwrap();
	}
	if !body.is_empty() {
		let image_path = storage_path.join(format!("{}.png", curr_playlist_id));
		if let Err(err) = fs::write(&image_path, body) {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(format!("Failed to save image: {}", err))
				.unwrap();
		}
	}
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
