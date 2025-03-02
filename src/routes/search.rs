use crate::core::app_state::AppState;
use crate::lobic_db::models::{Music, MusicResponse, Playlist, User, UserDataResponse};
use crate::schema::{music, users};
use axum::{
	extract::{Query, State},
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use strsim::jaro_winkler;

#[derive(Deserialize)]
pub struct SearchQuery {
	search_category: String,
	search_string: String,
}

#[derive(Serialize)]
pub struct SearchResponse {
	songs: Vec<MusicResponse>,
	people: Vec<UserDataResponse>,
	playlists: Vec<Playlist>,
}

pub async fn search(State(app_state): State<AppState>, Query(params): Query<SearchQuery>) -> Response<String> {
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

	let category = params.search_category.to_lowercase();
	let search_string = params.search_string.to_lowercase();
	let response = match category.as_str() {
		"title" | "album" | "artist" => {
			let all_music = match music::table.load::<Music>(&mut db_conn) {
				Ok(entries) => entries,
				Err(err) => {
					return Response::builder()
						.status(StatusCode::INTERNAL_SERVER_ERROR)
						.body(format!("Database error: {err}"))
						.unwrap();
				}
			};

			let search_results = all_music
				.into_iter()
				.map(|entry| {
					let (score, exact_match) = calculate_music_score(&entry, &category, &search_string);
					let weighted_score = if exact_match { 10000.0 } else { score };
					(entry, weighted_score)
				})
				.filter(|(_, score)| *score > 6.0)
				.collect::<Vec<_>>();

			let mut sorted_results = search_results;
			sorted_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

			// Convert Music entries to MusicResponse with image URLs
			let music_responses = sorted_results
				.into_iter()
				.map(|(entry, _)| Music::create_music_response(entry))
				.collect();

			SearchResponse {
				songs: music_responses,
				people: vec![],
				playlists: vec![],
			}
		}
		"people" => {
			let all_users = match users::table.load::<User>(&mut db_conn) {
				Ok(entries) => entries,
				Err(err) => {
					return Response::builder()
						.status(StatusCode::INTERNAL_SERVER_ERROR)
						.body(format!("Database error: {err}"))
						.unwrap();
				}
			};
			let search_results = all_users
				.into_iter()
				.map(|entry| {
					let (score, exact_match) = calculate_people_score(&entry, &search_string);
					let weighted_score = if exact_match { 10000.0 } else { score };
					(entry, weighted_score)
				})
				.filter(|(_, score)| *score > 6.0)
				.collect::<Vec<_>>();

			let mut sorted_results = search_results;
			sorted_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

			let people_response = sorted_results
				.into_iter()
				.map(|(entry, _)| UserDataResponse {
					user_id: entry.user_id,
					username: entry.username,
					email: entry.email,
				})
				.collect();

			SearchResponse {
				songs: vec![],
				people: people_response,
				playlists: vec![],
			}
		}
		_ => {
			return Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(format!("Unsupported search category: {}", category))
				.unwrap();
		}
	};

	// Serialize and return the response
	match serde_json::to_string(&response) {
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

fn calculate_music_score(entry: &Music, category: &str, search_string: &str) -> (f64, bool) {
	let search_term = search_string.to_lowercase();

	let contains_search_term = |field: &str| -> f64 { field.to_lowercase().contains(&search_term) as i32 as f64 * 8.0 };

	match category {
		"title" => {
			let exact = entry.title.eq_ignore_ascii_case(search_string);
			let similarity = jaro_winkler(&entry.title, search_string);
			let contains_bonus = contains_search_term(&entry.title) * 0.75;
			(similarity * 12.0 + contains_bonus, exact)
		}
		"album" => {
			let exact = entry.album.eq_ignore_ascii_case(search_string);
			let similarity = jaro_winkler(&entry.album, search_string);
			let contains_bonus = contains_search_term(&entry.album);
			(similarity * 6.0 + contains_bonus, exact)
		}
		"artist" => {
			let exact = entry.artist.eq_ignore_ascii_case(search_string);
			let similarity = jaro_winkler(&entry.artist, search_string);
			let contains_bonus = contains_search_term(&entry.artist);
			(similarity * 15.0 + contains_bonus, exact)
		}
		_ => (0.0, false),
	}
}

fn calculate_people_score(entry: &User, search_string: &str) -> (f64, bool) {
	let search_term = search_string.to_lowercase();
	let contains_search_term = |field: &str| -> f64 { field.to_lowercase().contains(&search_term) as i32 as f64 * 8.0 };
	let exact = entry.username.eq_ignore_ascii_case(search_string);
	let similarity = jaro_winkler(&entry.username, search_string);
	let contains_bonus = contains_search_term(&entry.username) * 0.75;
	(similarity * 12.0 + contains_bonus, exact)
}
