use crate::{
	core::app_state::AppState,
	lobic_db::models::{Music, MusicResponse},
	schema::{music, play_log},
};
use axum::{
	extract::{Query, State},
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::Deserialize;

// /music/get_recently_played?user_id=123&page_length=20
// /music/get_recently_played?user_id=123
#[derive(Debug, Deserialize)]
pub struct RecentlyPlayedQueryParams {
	pub user_id: String,
	#[serde(default)]
	pub start_index: i64, //defaults to 0
	pub page_length: Option<i64>,
}

pub async fn get_recently_played(
	State(app_state): State<AppState>,
	Query(params): Query<RecentlyPlayedQueryParams>,
) -> Response<String> {
	// Get a database connection
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(format!("Failed to get DB from pool: {err}"))
				.unwrap();
		}
	};

	let mut query = play_log::table
		.filter(play_log::user_id.eq(&params.user_id))
		.order(play_log::music_played_date_time.desc()) // Most recent first
		.inner_join(music::table)
		.select(music::all_columns)
		.offset(params.start_index)
		.into_boxed();
	if let Some(length) = params.page_length {
		if length > 0 {
			query = query.limit(length);
		}
	}
	match query.load::<Music>(&mut db_conn) {
		Ok(music_entries) => {
			if music_entries.is_empty() {
				return Response::builder()
					.status(StatusCode::NOT_FOUND)
					.body("No top tracks found".to_string())
					.unwrap();
			}

			let responses: Vec<MusicResponse> = music_entries
				.into_iter()
				.map(|entry| Music::create_music_response(entry))
				.collect();

			match serde_json::to_string(&responses) {
				Ok(json) => Response::builder()
					.status(StatusCode::OK)
					.header(header::CONTENT_TYPE, "application/json")
					.body(json)
					.unwrap(),
				Err(err) => Response::builder()
					.status(StatusCode::INTERNAL_SERVER_ERROR)
					.body(format!("Failed to serialize response: {err}"))
					.unwrap(),
			}
		}
		Err(err) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Database error: {err}"))
			.unwrap(),
	}
}
