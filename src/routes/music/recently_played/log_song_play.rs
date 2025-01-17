use crate::core::app_state::AppState;
use crate::lobic_db::models::PlayLog;
use axum::{extract::State, http::status::StatusCode, response::Response, Json};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogSongPlay {
	pub user_id: String,
	pub music_id: String,
}

pub async fn log_song_play(State(app_state): State<AppState>, Json(payload): Json<LogSongPlay>) -> Response<String> {
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(format!("Failed to get DB from pool: {err}"))
				.unwrap();
		}
	};

	use crate::schema::play_log::dsl::*;
	let curr_music_played_date_time = Utc::now().to_rfc3339();

	// Create a new PlayLog record
	let new_play_log = PlayLog {
		user_id: payload.user_id.clone(),
		music_id: payload.music_id.clone(),
		music_played_date_time: curr_music_played_date_time.clone(),
		user_times_played: 1, // Initialize user_times_played to 1
	};

	// Insert or update the play log
	match diesel::insert_into(play_log)
		.values(&new_play_log)
		.on_conflict((user_id, music_id)) // Handle conflict on (user_id, music_id)
		.do_update()
		.set((
			music_played_date_time.eq(curr_music_played_date_time),
			user_times_played.eq(user_times_played + 1), // Increment user_times_played
		))
		.execute(&mut db_conn)
	{
		Ok(_) => Response::builder()
			.status(StatusCode::CREATED)
			.body("Song play logged successfully".to_string())
			.unwrap(),
		Err(err) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Failed to log song play: {}", err))
			.unwrap(),
	}
}
