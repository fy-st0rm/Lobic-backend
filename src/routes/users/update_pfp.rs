use crate::config::USER_PFP_STORAGE;

use axum::{body::Bytes, extract::Query, http::StatusCode, response::Response};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct UserUuid {
	user_uuid: String,
}

pub async fn update_pfp(Query(user_uuid): Query<UserUuid>, body: Bytes) -> Response<String> {
	let user_uuid = match Uuid::parse_str(&user_uuid.user_uuid) {
		Ok(uuid) => uuid,
		Err(_) => {
			return Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body("Invalid UUID".to_string())
				.unwrap();
		}
	};

	let storage_path = Path::new(USER_PFP_STORAGE);
	if let Err(err) = fs::create_dir_all(storage_path) {
		return Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Failed to create directory: {}", err))
			.unwrap();
	}

	let image_path = storage_path.join(format!("{}.png", user_uuid));
	if let Err(err) = fs::write(&image_path, body) {
		return Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Failed to save image: {}", err))
			.unwrap();
	}

	Response::builder()
		.status(StatusCode::OK)
		.body("Profile picture updated successfully".to_string())
		.unwrap()
}
