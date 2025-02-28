use crate::core::app_state::AppState;
use axum::{
	extract::{Query, State},
	http::{header, StatusCode},
	response::Response,
};
use diesel::dsl::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AlbumQuery {
	#[serde(default)]
	start_index: i64,
	page_length: Option<i64>,
}

#[derive(Serialize)]
struct AlbumResponse {
	album: String,
	songs_count: i64,
	image_uuid: String,
}

fn generate_image_uuid(artist: &str, album: &str) -> String {
	let mut hasher = DefaultHasher::new();
	artist.hash(&mut hasher);
	album.hash(&mut hasher);
	let hash = hasher.finish();
	Uuid::from_u64_pair(hash, hash).to_string()
}

fn process_grouped_items(items: Vec<(String, String, i64)>) -> Vec<AlbumResponse> {
	let mut album_map: HashMap<String, (i64, String)> = HashMap::new();

	for (artist, album, count) in items {
		let entry = album_map
			.entry(album.clone())
			.or_insert((0, generate_image_uuid(&artist, &album)));
		entry.0 = count;
	}

	album_map
		.into_iter()
		.map(|(album, (songs_count, image_uuid))| AlbumResponse {
			album,
			songs_count,
			image_uuid,
		})
		.collect()
}

pub async fn browse_albums(State(app_state): State<AppState>, Query(params): Query<AlbumQuery>) -> Response<String> {
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

	// Modified query to count distinct music_ids for each album using a subquery
	let mut query = music
		.group_by((artist, album))
		.select((
			artist,
			album,
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
