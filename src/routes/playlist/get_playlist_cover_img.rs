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
use tokio::fs::File;
use tokio_util::io::ReaderStream;

pub async fn get_playlist_cover_img(Path(playlist_id): Path<String>) -> impl IntoResponse {
	let mut path = PathBuf::from(PLAYLIST_COVER_IMG_STORAGE);
	path.push(format!("{}.png", &playlist_id));

	let file = match File::open(&path).await {
		Ok(file) => file,
		Err(_) => {
			return Response::builder()
				.status(StatusCode::NOT_FOUND)
				.body(Body::from("Playlist cover image not found"))
				.unwrap();
		}
	};

	// Convert the file into a stream
	let stream = ReaderStream::new(file);
	// Convert the stream into a response body
	let body = Body::from_stream(stream);

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
		.header(header::CACHE_CONTROL, "public, max-age=31536000") // Add caching
		.body(body)
		.unwrap()
}
