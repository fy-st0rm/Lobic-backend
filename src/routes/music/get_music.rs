use axum::{
	extract::{Query, State},
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::Deserialize;
use std::{collections::hash_map::DefaultHasher, hash::Hash, hash::Hasher};
use uuid::Uuid;

use crate::{
	core::app_state::AppState,
	lobic_db::models::{Music, MusicResponse},
	schema::music::dsl::*,
};

#[derive(Deserialize)]
pub struct MusicQuery {
	title: Option<String>,
	uuid: Option<String>,
	artist: Option<String>,
	album: Option<String>,
	genre: Option<String>,
	#[serde(default)]
	start_index: i64,
	page_length: Option<i64>,
}

pub async fn get_music(State(app_state): State<AppState>, Query(params): Query<MusicQuery>) -> Response<String> {
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			let msg = format!("Failed to get DB from pool: {err}");
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(msg)
				.unwrap();
		}
	};

	let mut query = music.into_boxed();

	if let Some(title_val) = params.title {
		query = query.filter(title.eq(title_val));
	}
	if let Some(uuid_val) = params.uuid {
		query = query.filter(music_id.eq(uuid_val));
	}
	if let Some(artist_val) = params.artist {
		query = query.filter(artist.eq(artist_val));
	}
	if let Some(album_val) = params.album {
		query = query.filter(album.eq(album_val));
	}
	if let Some(genre_val) = params.genre {
		query = query.filter(genre.eq(genre_val));
	}

	query = query.offset(params.start_index);
	if let Some(length) = params.page_length {
		query = query.limit(length);
	}

	match query.load::<Music>(&mut db_conn) {
		Ok(music_entries) => {
			if music_entries.is_empty() {
				return Response::builder()
					.status(StatusCode::NOT_FOUND)
					.body("No music entries found".to_string())
					.unwrap();
			}

			let responses: Vec<MusicResponse> = music_entries
				.into_iter()
				.map(|entry| {
					// Generate image URL based on artist and album
					let mut hasher = DefaultHasher::new();
					entry.artist.hash(&mut hasher);
					entry.album.hash(&mut hasher);
					let hash = hasher.finish();
					let img_uuid = Uuid::from_u64_pair(hash, hash);

					MusicResponse {
						id: entry.music_id.clone(),
						artist: entry.artist,
						title: entry.title,
						album: entry.album,
						genre: entry.genre,
						times_played: entry.times_played,
						image_url: img_uuid.to_string(),
					}
				})
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
