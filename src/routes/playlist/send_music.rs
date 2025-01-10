use axum::{
	extract::Path,
	response::{IntoResponse, Response},
};
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncReadExt};

pub async fn send_music(Path(music_id): Path<String>, // Extract `path` from the URL path
) -> impl IntoResponse {
	// Open the file

	let mut path = PathBuf::from("storage/music_db");
	path.push(format!("{}.mp3", music_id));

	let mut file = match File::open(&path).await {
		Ok(file) => file,
		Err(_) => return (axum::http::StatusCode::NOT_FOUND, "File not found").into_response(),
	};

	// Read the file into a byte vector
	let mut file_bytes = Vec::new();
	if let Err(_) = file.read_to_end(&mut file_bytes).await {
		return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response();
	}

	// Serve the file as a response
	Response::builder()
		.header("Content-Type", "audio/mpeg")
		.header("Content-Disposition", "attachment; filename=\"music.mp3\"")
		.body(file_bytes.into())
		.unwrap()
}
