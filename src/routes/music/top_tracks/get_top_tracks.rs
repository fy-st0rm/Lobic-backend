use crate::core::app_state::AppState;
use axum::{
	extract::{Query, State},
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::config::{COVER_IMG_STORAGE, MUSIC_STORAGE};
use crate::schema::{music, play_log};

#[derive(Debug, Serialize)]
pub struct TopTracksResponse {
	pub id: String,
	pub filename: String,
	pub artist: String,
	pub title: String,
	pub album: String,
	pub genre: String,
	pub times_played: i32,
	pub cover_art_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TopTracksQueryParams {
	pub user_id: String,
	pub pagination_limit: Option<i64>,
}

pub async fn get_top_tracks(
	State(app_state): State<AppState>,
	Query(params): Query<TopTracksQueryParams>,
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

	// Fetch recently played songs for the user
	let limit = params.pagination_limit.unwrap_or(10); // Default to 10 if not provided
	let result = play_log::table
		.filter(play_log::user_id.eq(&params.user_id))
		.filter(play_log::user_times_played.ge(1)) // Only include records where user_times_played >= 1
		.order(play_log::user_times_played.desc()) // Order by most times listened
		.limit(limit) // Limit the number of results
		.inner_join(music::table)
		.select((
			music::music_id,
			music::artist,
			music::title,
			music::album,
			music::genre,
			music::times_played,
		))
		.load::<(String, String, String, String, String, i32)>(&mut db_conn);

	// Handle the query result
	match result {
		Ok(music_entries) => {
			if music_entries.is_empty() {
				return Response::builder()
					.status(StatusCode::NOT_FOUND)
					.body("No top tracks found".to_string())
					.unwrap();
			}

			// Map the database entries to the response format
			let responses: Vec<TopTracksResponse> = music_entries
				.into_iter()
				.map(|(music_id, artist, title, album, genre, times_played)| {
					let cover_art_path = format!("{}/{}.png", COVER_IMG_STORAGE, music_id);
					let has_cover = fs::metadata(&cover_art_path).is_ok();

					TopTracksResponse {
						id: music_id.clone(),
						filename: format!("{}/{}.mp3", MUSIC_STORAGE, music_id),
						artist,
						title,
						album,
						genre,
						times_played,
						cover_art_path: has_cover.then_some(cover_art_path),
					}
				})
				.collect();

			// Serialize the response and return it
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
