use crate::{
	core::app_state::AppState,
	lobic_db::models::{Music, MusicResponse},
};
use axum::{
	extract::{Query, State},
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::Deserialize;

use crate::schema::{liked_songs, music};

// /music/liked_song/get?user_id=123&start_index=10&page_length=20
// /music/liked_song/get?user_id=123&page_length=20
// /music/liked_song/get?user_id=123
#[derive(Debug, Deserialize)]
pub struct LikedSongsQueryParams {
	pub user_id: String,
	#[serde(default)]
	pub start_index: i64, //defaults to 0
	pub page_length: Option<i64>,
}

pub async fn get_liked_songs(
	State(app_state): State<AppState>,
	Query(params): Query<LikedSongsQueryParams>,
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

	let mut query = liked_songs::table
		.filter(liked_songs::user_id.eq(&params.user_id))
		.order(liked_songs::song_added_date_time.desc()) // Most recent first
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
