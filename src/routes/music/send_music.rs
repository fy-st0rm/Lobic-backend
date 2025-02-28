use axum::{
	body::Body,
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
};
use std::io;
use std::path::PathBuf;
use tokio::{fs::File, io::BufReader};
use tokio_util::io::ReaderStream;

use crate::config::MUSIC_STORAGE;
use crate::core::app_state::AppState;

pub async fn send_music(Path(curr_music_id): Path<String>, State(_app_state): State<AppState>) -> impl IntoResponse {
	// Validate music_id format first
	if !is_valid_music_id(&curr_music_id) {
		return (StatusCode::BAD_REQUEST, "Invalid music ID format").into_response();
	}

	// Open the file
	let mut path = PathBuf::from(MUSIC_STORAGE);
	path.push(format!("{}.mp3", curr_music_id));

	let file = match File::open(&path).await {
		Ok(file) => file,
		Err(err) => {
			let msg = match err.kind() {
				io::ErrorKind::NotFound => "File not found",
				io::ErrorKind::PermissionDenied => "Permission denied",
				_ => "Failed to open file",
			};
			return (StatusCode::NOT_FOUND, msg).into_response();
		}
	};

	// Create a buffered reader and convert it to a stream
	let reader = BufReader::new(file);
	let stream = ReaderStream::new(reader);
	let body = Body::from_stream(stream);

	// Build the response with appropriate headers
	match Response::builder()
		.header("Content-Type", "audio/mpeg")
		.header(
			"Content-Disposition",
			format!("attachment; filename=\"{}.mp3\"", curr_music_id),
		)
		.body(body)
	{
		Ok(response) => response.into_response(),
		Err(err) => {
			let msg = format!("Failed to build response: {}", err);
			(StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
		}
	}
}

// Helper function to validate music_id format
fn is_valid_music_id(id: &str) -> bool {
	!id.is_empty() && id.len() < 100 && id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}
