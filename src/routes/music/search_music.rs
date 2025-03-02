use crate::core::app_state::AppState;
use crate::lobic_db::models::Music;
use axum::{
	extract::{Query, State},
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::Deserialize;
use std::cmp::Ordering;
use strsim::jaro_winkler;

#[derive(Deserialize)]
pub struct SearchQuery {
	search_string: String,
	#[serde(default)]
	start_index: usize,
	page_length: Option<usize>,
}

pub async fn search_music(State(app_state): State<AppState>, Query(params): Query<SearchQuery>) -> Response<String> {
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

	// Fetch all music entries from the database
	let all_music = match music.load::<Music>(&mut db_conn) {
		Ok(entries) => entries,
		Err(err) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(format!("Database error: {err}"))
				.unwrap();
		}
	};

	// Perform fuzzy search on all fields with weighted scores
	let search_results = all_music
		.into_iter()
		.map(|entry| {
			// Check for exact matches in title, artist, or album
			let exact_match = entry.title.eq_ignore_ascii_case(&params.search_string)
				|| entry.artist.eq_ignore_ascii_case(&params.search_string)
				|| entry.album.eq_ignore_ascii_case(&params.search_string);

			// Calculate similarity scores for each field
			let title_score = jaro_winkler(&entry.title, &params.search_string);
			let artist_score = jaro_winkler(&entry.artist, &params.search_string);
			let album_score = jaro_winkler(&entry.album, &params.search_string);
			let genre_score = jaro_winkler(&entry.genre, &params.search_string);

			let weighted_score =
				if exact_match {
					10000.0 // Exact matches get highest priority
				} else {
					// Field similarity scores with weights
					let weighted_artist = artist_score * 15.0;
					let weighted_title = title_score * 12.0;
					let weighted_album = album_score * 6.0;
					let weighted_genre = genre_score * 2.0;

					//bonus weights for partial match in artist and title
					let search_term = params.search_string.to_lowercase();
					let contains_search_term = |field: &str| -> f64 {
						match field.to_lowercase().contains(&search_term) {
							true => 8.0,
							false => 0.0,
						}
					};

					let artist_contains_bonus = contains_search_term(&entry.artist);
					let title_contains_bonus = contains_search_term(&entry.title) * 0.75;

					// Sum all components
					weighted_artist
						+ weighted_title + weighted_album
						+ weighted_genre + artist_contains_bonus
						+ title_contains_bonus
				};

			(entry, weighted_score)
		})
		.filter(|(_, score)| *score > 6.0) // Filter out low similarity results
		.collect::<Vec<_>>();

	// Sort results by weighted score (descending order)
	let mut sorted_results = search_results;
	sorted_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

	// Apply pagination
	let paginated_results = sorted_results
		.into_iter()
		.skip(params.start_index)
		.take(params.page_length.unwrap_or(10))
		.map(|(entry, _)| Music::create_music_response(entry))
		.collect::<Vec<_>>();

	// Return the results as JSON
	match serde_json::to_string(&paginated_results) {
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
