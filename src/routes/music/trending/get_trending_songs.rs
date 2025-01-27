use axum::{
	extract::{Query, State},
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::Deserialize;

use crate::core::app_state::AppState;
use crate::lobic_db::models::MusicResponse;

use crate::schema::music;

#[derive(Debug, Deserialize)]
pub struct TrendingSongsQueryParams {
	#[serde(default)]
	pub start_index: i64, //defaults to 0
	pub page_length: Option<i64>,
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
	let mut query = music::table
		.select((
			music::music_id,
			music::artist,
			music::title,
			music::album,
			music::genre,
			music::times_played,
		))
		.order(music::times_played.desc())
		.offset(params.start_index)
		.into_boxed();

	// Apply page length if specified
	if let Some(length) = params.page_length {
		if length > 0 {
			query = query.limit(length);
		}
		//else infinity
	}
	let result = query.load::<(String, String, String, String, String, i32)>(&mut db_conn);

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
			let responses: Vec<MusicResponse> = music_entries
				.into_iter()
				.map(|(music_id, artist, title, album, genre, times_played)| MusicResponse {
					id: music_id.clone(),
					artist: artist,
					title: title,
					album: album,
					genre: genre,
					times_played: times_played,
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
