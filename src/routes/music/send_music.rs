use axum::{
	body::Body,
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
};
use diesel::{result::Error as DieselError, Connection, RunQueryDsl};
use diesel::{update, ExpressionMethods, QueryDsl};
use std::io;
use std::path::PathBuf;
use tokio::{fs::File, io::BufReader};
use tokio_util::io::ReaderStream;

use crate::{core::app_state::AppState, schema::music::dsl::*};

pub async fn send_music(Path(curr_music_id): Path<String>, State(app_state): State<AppState>) -> impl IntoResponse {
	// Validate music_id format first
	if !is_valid_music_id(&curr_music_id) {
		return (StatusCode::BAD_REQUEST, "Invalid music ID format").into_response();
	}

	// Open the file
	let mut path = PathBuf::from("storage/music_db");
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

	// Get database connection from pool
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			let msg = format!("Database connection error: {}", err);
			return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response();
		}
	};

	// Update play count with retry logic
	const MAX_RETRIES: u32 = 3;
	let mut retries = 0;
	let update_result = loop {
		match db_conn.transaction(|conn| {
			update(music.filter(music_id.eq(&curr_music_id)))
				.set(times_played.eq(times_played + 1))
				.execute(conn)
		}) {
			Ok(result) => break Ok(result),
			Err(DieselError::DatabaseError(diesel::result::DatabaseErrorKind::Unknown, _)) if retries < MAX_RETRIES => {
				retries += 1;
				tokio::time::sleep(tokio::time::Duration::from_millis(10 * retries as u64)).await;
				continue;
			}
			Err(err) => break Err(err),
		}
	};

	if let Err(err) = update_result {
		let msg = format!("Database update error: {}", err);
		return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response();
	}

	// Create a buffered reader and convert it to a stream
	let reader = BufReader::new(file);
	let stream = ReaderStream::new(reader);
	let body = Body::from_stream(stream);

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
