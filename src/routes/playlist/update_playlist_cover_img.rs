use crate::config::PLAYLIST_COVER_IMG_STORAGE;

use axum::{body::Bytes, extract::Query, http::StatusCode, response::Response};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PlaylistId {
	playlist_id: String,
}

pub async fn update_playlist_cover_img(Query(playlist_id): Query<PlaylistId>, body: Bytes) -> Response<String> {
	let uuid = match Uuid::parse_str(&playlist_id.playlist_id) {
		Ok(uuid) => uuid,
		Err(_) => {
			return Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body("Invalid UUID".to_string())
				.unwrap();
		}
	};

	let storage_path = Path::new(PLAYLIST_COVER_IMG_STORAGE);
	if let Err(err) = fs::create_dir_all(storage_path) {
		return Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Failed to create directory: {}", err))
			.unwrap();
	}

	let image_path = storage_path.join(format!("{}.png", uuid));
	if let Err(err) = fs::write(&image_path, body) {
		return Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Failed to save image: {}", err))
			.unwrap();
	}

	Response::builder()
		.status(StatusCode::OK)
		.body("Cover image updated successfully".to_string())
		.unwrap()
}
