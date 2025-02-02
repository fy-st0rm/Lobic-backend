use crate::core::app_state::AppState;
use axum::{
	extract::{Path, State},
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Serialize)]
struct ItemWithMetadata {
	name: String,
	count: i64,
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

fn process_grouped_items(items: Vec<(String, String)>) -> Vec<ItemWithMetadata> {
	let mut artist_map: HashMap<String, (i64, Vec<String>)> = HashMap::new();

	for (name, sub_name) in items {
		let entry = artist_map.entry(name.clone()).or_insert((0, Vec::new()));
		entry.0 += 1; // Increment count
		if entry.1.len() < 4 {
			entry.1.push(generate_image_uuid(&name, &sub_name));
		}
	}
	artist_map
		.into_iter()
		.map(|(name, (count, image_uuids))| ItemWithMetadata {
			name,
			count,
			image_uuids,
		})
		.collect()
}

pub async fn browse_all(State(app_state): State<AppState>, Path(category): Path<String>) -> Response<String> {
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

	let result = match category.as_str() {
		"artists" => music
			.select(artist)
			.distinct()
			.select((artist, album))
			.load::<(String, String)>(&mut db_conn)
			.map(process_grouped_items),

		"albums" => music
			.select(album)
			.distinct() // Apply distinct only on the album column
			.select((artist, album))
			.load::<(String, String)>(&mut db_conn)
			.map(process_grouped_items),

		"genres" => music
			.select(genre)
			.distinct()
			.select((artist, album))
			.load::<(String, String)>(&mut db_conn)
			.map(process_grouped_items),

		_ => {
			return Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body("Invalid category. Use 'artists', 'albums', or 'genres'.".to_string())
				.unwrap();
		}
	};

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
