use crate::core::app_state::AppState;
use crate::lobic_db::models::{Music, MusicResponse, Playlist, PlaylistInfo, User, UserDataResponse};
use crate::schema::{music, playlists, users};
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
	playlists: Vec<PlaylistInfo>,
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
		"all" => {
			// Define a constant limit for all searches
			const SEARCH_LIMIT: i64 = 10;

			// Search music with limit
			let music_results = music::table
				.filter(
					music::title
						.like(format!("%{}%", search_string))
						.or(music::album.like(format!("%{}%", search_string)))
						.or(music::artist.like(format!("%{}%", search_string))),
				)
				.limit(SEARCH_LIMIT)
				.load::<Music>(&mut db_conn)
				.map(|entries| {
					entries
						.into_iter()
						.map(Music::create_music_response)
						.collect::<Vec<_>>()
				})
				.unwrap_or_else(|_| vec![]);

			// Search users with limit
			let people_results = users::table
				.filter(users::username.like(format!("%{}%", search_string)))
				.limit(SEARCH_LIMIT)
				.load::<User>(&mut db_conn)
				.map(|entries| {
					entries
						.into_iter()
						.map(|entry| UserDataResponse {
							user_id: entry.user_id,
							username: entry.username,
							email: entry.email,
						})
						.collect::<Vec<_>>()
				})
				.unwrap_or_else(|_| vec![]);

			// Search playlists with limit
			let playlist_results = playlists::table
				.filter(playlists::playlist_name.like(format!("%{}%", search_string)))
				.limit(SEARCH_LIMIT)
				.load::<Playlist>(&mut db_conn)
				.unwrap_or_else(|_| vec![]);
			let playlists_response = playlist_results
				.into_iter()
				.map(|playlist| PlaylistInfo {
					playlist_id: playlist.playlist_id,
					user_id: playlist.user_id,
					playlist_name: playlist.playlist_name,
					creation_date_time: playlist.creation_date_time,
					last_updated_date_time: playlist.last_updated_date_time,
					is_playlist_combined: playlist.is_playlist_combined,
				})
				.collect();

			SearchResponse {
				songs: music_results,
				people: people_results,
				playlists: playlists_response,
			}
		}
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
				.filter(|(_, score)| *score > 9.0)
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
				.filter(|(_, score)| *score > 12.0)
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
		"playlists" => {
			let all_playlists = match playlists::table.load::<Playlist>(&mut db_conn) {
				Ok(entries) => entries,
				Err(err) => {
					return Response::builder()
						.status(StatusCode::INTERNAL_SERVER_ERROR)
						.body(format!("Database error: {err}"))
						.unwrap();
				}
			};
			let search_results = all_playlists
				.into_iter()
				.map(|entry| {
					let (score, exact_match) = calculate_playlist_score(&entry, &search_string);
					let weighted_score = if exact_match { 10000.0 } else { score };
					(entry, weighted_score)
				})
				.filter(|(_, score)| *score > 6.0)
				.collect::<Vec<_>>();

			let mut sorted_results = search_results;
			sorted_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

			let playlist_response = sorted_results
				.into_iter()
				.map(|(entry, _)| PlaylistInfo {
					playlist_id: entry.playlist_id,
					user_id: entry.user_id,
					playlist_name: entry.playlist_name,
					creation_date_time: entry.creation_date_time,
					last_updated_date_time: entry.last_updated_date_time,
					is_playlist_combined: entry.is_playlist_combined,
				})
				.collect();

			SearchResponse {
				songs: vec![],
				people: vec![],
				playlists: playlist_response,
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

fn calculate_playlist_score(entry: &Playlist, search_string: &str) -> (f64, bool) {
	let search_term = search_string.to_lowercase();
	let contains_search_term = |field: &str| -> f64 { field.to_lowercase().contains(&search_term) as i32 as f64 * 8.0 };
	let exact = entry.playlist_name.eq_ignore_ascii_case(search_string);
	let similarity = jaro_winkler(&entry.playlist_name, search_string);
	let contains_bonus = contains_search_term(&entry.playlist_name) * 0.75;
	(similarity * 12.0 + contains_bonus, exact)
}
