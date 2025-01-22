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
use crate::schema::{liked_songs, music};

#[derive(Debug, Serialize)]
pub struct LikedSongsResponse {
	pub id: String,
	pub filename: String,
	pub artist: String,
	pub title: String,
	pub album: String,
	pub genre: String,
	pub times_played: i32,
	pub cover_art_path: Option<String>,
}

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

	// Build the query
	let mut query = liked_songs::table
		.filter(liked_songs::user_id.eq(&params.user_id))
		.inner_join(music::table)
		.order(liked_songs::song_added_date_time.desc()) // Most recent first
		.select((
			music::music_id,
			music::artist,
			music::title,
			music::album,
			music::genre,
			music::times_played,
		))
		.offset(params.start_index)
		.into_boxed();

	// Apply page length if specified
	if let Some(length) = params.page_length {
		if length > 0 {
			query = query.limit(length);
		}
		//else infinity
	}

	// Execute the query
	let result = query.load::<(String, String, String, String, String, i32)>(&mut db_conn);

	// Handle the query result
	match result {
		Ok(music_entries) => {
			if music_entries.is_empty() {
				return Response::builder()
					.status(StatusCode::NOT_FOUND)
					.body("No liked songs found".to_string())
					.unwrap();
			}

			// Map the database entries to the response format
			let responses: Vec<LikedSongsResponse> = music_entries
				.into_iter()
				.map(|(music_id, artist, title, album, genre, times_played)| {
					let cover_art_path = format!("{}/{}.png", COVER_IMG_STORAGE, music_id);
					let has_cover = fs::metadata(&cover_art_path).is_ok();

					LikedSongsResponse {
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
