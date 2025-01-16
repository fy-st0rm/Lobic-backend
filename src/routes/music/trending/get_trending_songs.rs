use axum::{
	extract::{Query, State},
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::app_state::AppState;
use crate::lobic_db::models::Music;
use crate::schema::music::dsl::*;

#[derive(Debug, Serialize)]
pub struct TrendingSongsResponse {
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
pub struct TrendingSongsQueryParams {
	pub pagination_limit: Option<i64>,
}

pub async fn get_trending_songs(
	State(app_state): State<AppState>,
	Query(params): Query<TrendingSongsQueryParams>,
) -> Response<String> {
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(format!("Failed to get DB from pool: {err}"))
				.unwrap();
		}
	};

	//Fetch the most played songs with pagination
	let limit = params.pagination_limit.unwrap_or(30); // Default to 30 if not provided
	let result = music
		.order(times_played.desc())
		.limit(limit)
		.load::<Music>(&mut db_conn);

	//Handle the query result
	match result {
		Ok(music_entries) => {
			if music_entries.is_empty() {
				return Response::builder()
					.status(StatusCode::NOT_FOUND)
					.body("No trending songs found".to_string())
					.unwrap();
			}

			//Map the database entries to the response format
			let responses: Vec<TrendingSongsResponse> = music_entries
				.into_iter()
				.map(|entry| {
					let cover_art_path = format!("{}/{}.png", crate::config::COVER_IMG_STORAGE, entry.music_id);
					let has_cover = std::fs::metadata(&cover_art_path).is_ok();

					TrendingSongsResponse {
						id: entry.music_id.clone(),
						filename: format!("{}/{}.mp3", crate::config::MUSIC_STORAGE, entry.music_id),
						artist: entry.artist,
						title: entry.title,
						album: entry.album,
						genre: entry.genre,
						times_played: entry.times_played,
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
