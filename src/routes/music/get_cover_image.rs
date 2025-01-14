use crate::config::COVER_IMG_STORAGE;

use axum::{
	body::Body,
	extract::Path,
	http::{header, StatusCode},
	response::{IntoResponse, Response},
};
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncReadExt};

pub async fn get_cover_image(Path(filename): Path<String>) -> impl IntoResponse {
	// Construct the path to the cover image
	let mut path = PathBuf::from(COVER_IMG_STORAGE);
	path.push(&filename);

	// Open the file
	let mut file = match File::open(&path).await {
		Ok(file) => file,
		Err(_) => {
			return (StatusCode::NOT_FOUND, "Image not found").into_response();
		}
	};

	// Read the file into a byte vector
	let mut file_bytes = Vec::new();
	if let Err(_) = file.read_to_end(&mut file_bytes).await {
		return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read image file").into_response();
	}

	// Determine the MIME type based on the file extension
	let mime_type = match path.extension().and_then(|ext| ext.to_str()) {
		Some("jpg") | Some("jpeg") => "image/jpeg",
		Some("png") => "image/png",
		Some("gif") => "image/gif",
		Some("webp") => "image/webp",
		_ => "application/octet-stream", // Fallback MIME type
	};

	// Serve the file as a response using Body
	Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, mime_type)
		.body(Body::from(file_bytes)) // Use Body::from to create the response body
		.unwrap()
}
