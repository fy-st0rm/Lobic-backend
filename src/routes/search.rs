// use crate::core::app_state::AppState;
// use crate::lobic_db::models::Music;
// use crate::lobic_db::models::MusicResponse;
// use axum::{
// 	extract::{Query, State},
// 	http::{header, StatusCode},
// 	response::Response,
// };
// use diesel::prelude::*;
// use serde::Deserialize;
// use std::cmp::Ordering;
// use strsim::jaro_winkler;

// use std::{collections::hash_map::DefaultHasher, hash::Hash, hash::Hasher};
// use uuid::Uuid;

// #[derive(Deserialize)]
// // seach_category = all | title | album | artist | genre | playlist | user
// pub struct SearchQuery {
// 	search_category: String,
// 	search_string: String,
// }
// pub struct SearchResponse {
// 	songs: Vec<MusicResponse>,
// 	people: Vec<(id : String ,
// 	username : String,
// email;)>
// }

// pub async fn search(State(app_state): State<AppState>, Query(params): Query<SearchQuery>) -> Response<String> {
// 	let mut db_conn = match app_state.db_pool.get() {
// 		Ok(conn) => conn,
// 		Err(err) => {
// 			let msg = format!("Failed to get DB from pool: {err}");
// 			return Response::builder()
// 				.status(StatusCode::INTERNAL_SERVER_ERROR)
// 				.body(msg)
// 				.unwrap();
// 		}
// 	};

// 	use crate::schema::music::dsl::*;

// 	// Fetch all music entries from the database
// 	let all_music = match music.load::<Music>(&mut db_conn) {
// 		Ok(entries) => entries,
// 		Err(err) => {
// 			return Response::builder()
// 				.status(StatusCode::INTERNAL_SERVER_ERROR)
// 				.body(format!("Database error: {err}"))
// 				.unwrap();
// 		}
// 	};

// 	// Perform fuzzy search on all fields with weighted scores
// 	let search_results = all_music
// 		.into_iter()
// 		.map(|entry| {
// 			// Check for exact matches in title, artist, or album
// 			let exact_match = entry.title.eq_ignore_ascii_case(&params.search_string)
// 				|| entry.artist.eq_ignore_ascii_case(&params.search_string)
// 				|| entry.album.eq_ignore_ascii_case(&params.search_string);

// 			// Calculate similarity scores for each field
// 			let title_score = jaro_winkler(&entry.title, &params.search_string);
// 			let artist_score = jaro_winkler(&entry.artist, &params.search_string);
// 			let album_score = jaro_winkler(&entry.album, &params.search_string);
// 			let genre_score = jaro_winkler(&entry.genre, &params.search_string);
// 		})
// 		.filter(|(_, score)| *score > 6.0) // Filter out low similarity results
// 		.collect::<Vec<_>>();

// 	// Sort results by weighted score (descending order)
// 	let mut sorted_results = search_results;
// 	sorted_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

// 	// Return the results as JSON
// 	match serde_json::to_string(&sorted_results) {
// 		Ok(json) => Response::builder()
// 			.status(StatusCode::OK)
// 			.header(header::CONTENT_TYPE, "application/json")
// 			.body(json)
// 			.unwrap(),
// 		Err(err) => Response::builder()
// 			.status(StatusCode::INTERNAL_SERVER_ERROR)
// 			.body(format!("Failed to serialize response: {err}"))
// 			.unwrap(),
// 	}
// }
