use crate::core::app_state::AppState;
use crate::lobic_db::models::PlayLog;
use axum::{extract::State, http::status::StatusCode, response::Response, Json};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::log::error;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogSongPlay {
	pub user_id: String,
	pub music_id: String,
}

pub async fn log_song_play(State(app_state): State<AppState>, Json(payload): Json<LogSongPlay>) -> Response<String> {
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			error!("Failed to get DB from pool: {}", err);
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(format!("Failed to get DB from pool: {}", err))
				.unwrap();
		}
	};

	use crate::schema::{music, play_log};
	match db_conn.transaction::<_, diesel::result::Error, _>(|conn| {
		let curr_music_played_date_time = Utc::now().to_rfc3339();

		let new_play_log = PlayLog {
			user_id: payload.user_id.clone(),
			music_id: payload.music_id.clone(),
			music_played_date_time: curr_music_played_date_time.clone(),
			user_times_played: 1,
		};

		diesel::insert_into(play_log::table)
			.values(&new_play_log)
			.on_conflict((play_log::user_id, play_log::music_id))
			.do_update()
			.set((
				play_log::music_played_date_time.eq(curr_music_played_date_time),
				play_log::user_times_played.eq(play_log::user_times_played + 1),
			))
			.execute(conn)?;

		diesel::update(music::table)
			.filter(music::music_id.eq(&payload.music_id))
			.set(music::times_played.eq(music::times_played + 1))
			.execute(conn)?;

		Ok(())
	}) {
		Ok(_) => Response::builder()
			.status(StatusCode::CREATED)
			.body("Song play logged successfully".to_string())
			.unwrap(),
		Err(err) => {
			error!("Failed to log song play: {}", err);
			Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(format!("Failed to log song play: {}", err))
				.unwrap()
		}
	}
}
