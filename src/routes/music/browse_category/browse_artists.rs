use crate::core::app_state::AppState;
use axum::{
	extract::{Query, State},
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ArtistQuery {
	#[serde(default)]
	start_index: i64,
	page_length: Option<i64>,
}

#[derive(Serialize)]
struct ArtistsResponse {
	artist: String,
	songs_count: i64,
	image_uuids: Vec<String>,
}

fn generate_image_uuid(artist: &str, album: &str) -> String {
	let mut hasher = DefaultHasher::new();
	artist.hash(&mut hasher);
	album.hash(&mut hasher);
	let hash = hasher.finish();
	Uuid::from_u64_pair(hash, hash).to_string()
}

fn process_grouped_items(items: Vec<(String, Vec<String>, i64)>) -> Vec<ArtistsResponse> {
	items
		.into_iter()
		.map(|(artist, albums, count)| {
			let image_uuids: Vec<String> = albums
				.iter()
				.take(4)
				.map(|album| generate_image_uuid(&artist, album))
				.collect();

			ArtistsResponse {
				artist,
				songs_count: count,
				image_uuids,
			}
		})
		.collect()
}

pub async fn browse_artists(State(app_state): State<AppState>, Query(params): Query<ArtistQuery>) -> Response<String> {
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

	use crate::schema::music::dsl::*;
	use diesel::dsl::sql;

	let mut query = music
		.group_by(artist)
		.select((
			artist,
			sql("GROUP_CONCAT(DISTINCT album)").into_sql::<diesel::sql_types::Text>(),
			sql("COUNT(DISTINCT music_id)").into_sql::<diesel::sql_types::BigInt>(),
		))
		.into_boxed();

	query = query.offset(params.start_index);

	if let Some(length) = params.page_length {
		if length > 0 {
			query = query.limit(length);
		}
	}

	let result = query
		.load::<(String, String, i64)>(&mut db_conn)
		.map(|items| {
			// Convert the concatenated albums string to a Vec<String>
			items
				.into_iter()
				.map(|(_artist, albums_str, count)| {
					let albums = albums_str.split(',').map(String::from).collect();
					(_artist, albums, count)
				})
				.collect()
		})
		.map(process_grouped_items);

	match result {
		Ok(items) => match serde_json::to_string(&items) {
			Ok(json) => Response::builder()
				.status(StatusCode::OK)
				.header(header::CONTENT_TYPE, "application/json")
				.body(json)
				.unwrap(),
			Err(err) => Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(format!("Failed to serialize response: {err}"))
				.unwrap(),
		},
		Err(err) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Database error: {err}"))
			.unwrap(),
	}
}
