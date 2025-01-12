use axum::{
	extract::{Path, State},
	response::{IntoResponse, Response},
};
use diesel::{update, ExpressionMethods, QueryDsl, RunQueryDsl};
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncReadExt};

use crate::{core::app_state::AppState, schema::music::dsl::*};

pub async fn send_music(Path(curr_music_id): Path<String>, State(app_state): State<AppState>) -> impl IntoResponse {
	// Open the file
	let mut path = PathBuf::from("storage/music_db");
	path.push(format!("{}.mp3", curr_music_id));

	let mut file = match File::open(&path).await {
		Ok(file) => file,
		Err(_) => return (axum::http::StatusCode::NOT_FOUND, "File not found").into_response(),
	};

	//increment the times_played of the current song
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			let msg = format!("Failed to get DB from pool: {err}");
			return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg).into_response();
		}
	};
	update(music.filter(music_id.eq(&curr_music_id)))
		.set(times_played.eq(times_played + 1))
		.execute(&mut db_conn)
		.unwrap();

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
