use crate::config::COVER_IMG_STORAGE;
use axum::{
	extract::Path,
	http::{header, StatusCode},
	response::Response,
};
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncReadExt};

pub async fn get_cover_image(Path(img_uuid): Path<String>) -> Response<axum::body::Body> {
	let filename = format!("{img_uuid}.png");
	let mut path = PathBuf::from(COVER_IMG_STORAGE);
	path.push(&filename);

	match File::open(&path).await {
		Ok(mut file) => {
			let mut file_bytes = Vec::new();
			if let Err(_) = file.read_to_end(&mut file_bytes).await {
				return serve_default_image().await;
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
		Err(_) => serve_default_image().await,
	}
}

async fn serve_default_image() -> Response<axum::body::Body> {
	let default_path = PathBuf::from("assets/default_music_cover.png");
	match File::open(&default_path).await {
		Ok(mut default_file) => {
			let mut default_bytes = Vec::new();
			if let Err(_) = default_file.read_to_end(&mut default_bytes).await {
				return Response::builder()
					.status(StatusCode::INTERNAL_SERVER_ERROR)
					.body(axum::body::Body::from("Failed to read default image file"))
					.unwrap();
			}

			Response::builder()
				.status(StatusCode::OK)
				.header(header::CONTENT_TYPE, "image/png")
				.body(axum::body::Body::from(default_bytes))
				.unwrap()
		}
		Err(_) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(axum::body::Body::from("Default image not found"))
			.unwrap(),
	}
}
