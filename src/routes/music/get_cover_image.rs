use axum::{
	extract::{Path, State},
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;
use std::{collections::hash_map::DefaultHasher, hash::Hash, hash::Hasher, path::PathBuf};
use tokio::{fs::File, io::AsyncReadExt};
use uuid::Uuid;

use crate::config::COVER_IMG_STORAGE;
use crate::{core::app_state::AppState, lobic_db::models::Music};

pub async fn get_cover_image(Path(id): Path<String>, State(app_state): State<AppState>) -> Response<axum::body::Body> {
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			let msg = format!("Failed to get DB from pool: {err}");
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(axum::body::Body::from(msg))
				.unwrap();
		}
	};

	use crate::schema::music::dsl::*;

	let result = music.filter(music_id.eq(&id)).first::<Music>(&mut db_conn);

	match result {
		Ok(music_entry) => {
			let mut hasher = DefaultHasher::new();
			music_entry.artist.hash(&mut hasher);
			music_entry.album.hash(&mut hasher);
			let hash = hasher.finish();
			let img_uuid = Uuid::from_u64_pair(hash, hash);

			let filename = format!("{}.png", img_uuid);

			let mut path = PathBuf::from(COVER_IMG_STORAGE);
			path.push(&filename);

			let mut file = match File::open(&path).await {
				Ok(file) => file,
				Err(_) => {
					return Response::builder()
						.status(StatusCode::NOT_FOUND)
						.body(axum::body::Body::from("Image not found"))
						.unwrap();
				}
			};

			let mut file_bytes = Vec::new();
			if let Err(_) = file.read_to_end(&mut file_bytes).await {
				return Response::builder()
					.status(StatusCode::INTERNAL_SERVER_ERROR)
					.body(axum::body::Body::from("Failed to read image file"))
					.unwrap();
			}

			let mime_type = match path.extension().and_then(|ext| ext.to_str()) {
				Some("jpg") | Some("jpeg") => "image/jpeg",
				Some("png") => "image/png",
				Some("gif") => "image/gif",
				Some("webp") => "image/webp",
				_ => "application/octet-stream",
			};

			Response::builder()
				.status(StatusCode::OK)
				.header(header::CONTENT_TYPE, mime_type)
				.body(axum::body::Body::from(file_bytes))
				.unwrap()
		}
		Err(err) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(axum::body::Body::from(format!("Database error: {err}")))
			.unwrap(),
	}
}
