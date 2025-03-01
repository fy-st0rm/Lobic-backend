use crate::{
	core::app_state::AppState,
	lobic_db::models::PlayLog,
	schema::{music, play_log},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::log::error;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogSongPlay {
	pub user_id: String,
	pub music_id: String,
}

const MAX_RETRIES: u32 = 3;

pub async fn log_song_play(State(app_state): State<AppState>, Json(payload): Json<LogSongPlay>) -> impl IntoResponse {
	// Get database connection from pool
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			error!("Failed to get DB from pool: {}", err);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Database connection error: {}", err),
			)
				.into_response();
		}
	};

	// Retry logic for the combined transaction
	let mut retries = 0;
	let transaction_result = loop {
		match db_conn.transaction::<_, diesel::result::Error, _>(|conn| {
			let curr_music_played_date_time = Utc::now().to_rfc3339();

			// Create new play log entry
			let new_play_log = PlayLog {
				user_id: payload.user_id.clone(),
				music_id: payload.music_id.clone(),
				music_played_date_time: curr_music_played_date_time.clone(),
				user_times_played: 1,
			};

			// Update play log
			diesel::insert_into(play_log::table)
				.values(&new_play_log)
				.on_conflict((play_log::user_id, play_log::music_id))
				.do_update()
				.set((
					play_log::music_played_date_time.eq(curr_music_played_date_time),
					play_log::user_times_played.eq(play_log::user_times_played + 1),
				))
				.execute(conn)?;

			// Update global play count
			diesel::update(music::table)
				.filter(music::music_id.eq(&payload.music_id))
				.set(music::times_played.eq(music::times_played + 1))
				.execute(conn)?;

			Ok(())
		}) {
			Ok(result) => break Ok(result),
			Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::Unknown, _))
				if retries < MAX_RETRIES =>
			{
				retries += 1;
				tokio::time::sleep(tokio::time::Duration::from_millis(10 * retries as u64)).await;
				continue;
			}
			Err(err) => break Err(err),
		}
	};

	match transaction_result {
		Ok(_) => (StatusCode::CREATED, "Song play logged successfully").into_response(),
		Err(err) => {
			error!("Failed to log song play: {}", err);
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Failed to log song play: {}", err),
			)
				.into_response()
		}
	}
}
