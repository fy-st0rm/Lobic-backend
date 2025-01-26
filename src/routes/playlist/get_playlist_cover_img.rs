use crate::config::PLAYLIST_COVER_IMG_STORAGE;

use axum::{
	body::Body,
	extract::Path,
	http::{
		header::{self},
		StatusCode,
	},
	response::{IntoResponse, Response},
};
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncReadExt};

pub async fn get_playlist_cover_img(Path(playlist_id): Path<String>) -> impl IntoResponse {
	let mut path = PathBuf::from(PLAYLIST_COVER_IMG_STORAGE);
	path.push(format!("{}.png", &playlist_id));

	let mut file = match File::open(&path).await {
		Ok(file) => file,
		Err(_) => {
			return serve_default_playlist_cover().await;
		}
	};

	let mut file_bytes = Vec::new();
	if let Err(_) = file.read_to_end(&mut file_bytes).await {
		return serve_default_playlist_cover().await;
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
		.body(Body::from(file_bytes))
		.unwrap()
}

async fn serve_default_playlist_cover() -> Response<Body> {
	let default_path = PathBuf::from("assets/default_playlist_cover.png");

	let mut default_file = match File::open(&default_path).await {
		Ok(file) => file,
		Err(_) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(Body::from("Default playlist cover not found"))
				.unwrap();
		}
	};

	let mut default_bytes = Vec::new();
	if let Err(_) = default_file.read_to_end(&mut default_bytes).await {
		return Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(Body::from("Failed to read default playlist cover file"))
			.unwrap();
	}

	Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "image/png")
		.body(Body::from(default_bytes))
		.unwrap()
}
