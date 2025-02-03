use crate::core::app_state::AppState;
use axum::{
	extract::State,
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use uuid::Uuid;
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
use std::collections::HashMap;
fn process_grouped_items(items: Vec<(String, String)>) -> Vec<ArtistsResponse> {
	let mut artist_map: HashMap<String, (i64, Vec<String>)> = HashMap::new();
	for (name, sub_name) in items {
		let entry = artist_map.entry(name.clone()).or_insert((0, Vec::new()));
		entry.0 += 1;
		if entry.1.len() < 4 {
			entry.1.push(generate_image_uuid(&name, &sub_name));
		}
	}
	artist_map
		.into_iter()
		.map(|(artist, (songs_count, image_uuids))| ArtistsResponse {
			artist,
			songs_count,
			image_uuids,
		})
		.collect()
}

pub async fn browse_artists(State(app_state): State<AppState>) -> Response<String> {
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

	let result = music
		.select(artist)
		.distinct()
		.select((artist, album))
		.load::<(String, String)>(&mut db_conn)
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
