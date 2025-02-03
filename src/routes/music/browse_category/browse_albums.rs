use crate::core::app_state::AppState;
use axum::{
	extract::State,
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

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

fn process_grouped_items(items: Vec<(String, String)>) -> Vec<AlbumResponse> {
	let mut album_map: HashMap<String, (i64, String)> = HashMap::new();
	for (artist, album) in items {
		let entry = album_map
			.entry(album.clone())
			.or_insert((0, generate_image_uuid(&artist, &album)));
		entry.0 += 1;
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

pub async fn browse_albums(State(app_state): State<AppState>) -> Response<String> {
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
		.select((artist, album))
		.distinct()
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
