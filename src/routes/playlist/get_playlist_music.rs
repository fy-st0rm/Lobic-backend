use crate::core::app_state::AppState;
use axum::{
	body::Body,
	extract::{Query, State},
	http::{header, status::StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::{collections::hash_map::DefaultHasher, hash::Hasher};
use uuid::Uuid;

#[derive(Queryable)]
struct MusicQueryResult {
	music_id: String,
	artist: String,
	title: String,
	album: String,
	genre: String,
	duration: i64,
	song_added_date_time: String,
	song_adder_id: String,
}

#[derive(Debug, Serialize)]
pub struct PlaylistMusicResponse {
	pub music_id: String,
	pub artist: String,
	pub title: String,
	pub album: String,
	pub genre: String,
	pub duration: i64,
	pub image_url: String,
	pub song_added_date_time: String,
	pub song_adder_id: String,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistQueryParams {
	pub playlist_id: String,
}

use crate::lobic_db::models::Playlist;
#[derive(Debug, Serialize)]
pub struct PlaylistDetailsResponse {
	pub playlist: Playlist,
	pub songs: Vec<PlaylistMusicResponse>,
}

impl PlaylistMusicResponse {
	fn from_query_result(result: MusicQueryResult) -> Self {
		let mut hasher = DefaultHasher::new();
		result.artist.hash(&mut hasher);
		result.album.hash(&mut hasher);
		let hash = hasher.finish();
		let img_uuid = Uuid::from_u64_pair(hash, hash);

		PlaylistMusicResponse {
			music_id: result.music_id,
			artist: result.artist,
			title: result.title,
			album: result.album,
			genre: result.genre,
			duration: result.duration,
			image_url: img_uuid.to_string(),
			song_added_date_time: result.song_added_date_time,
			song_adder_id: result.song_adder_id,
		}
	}
}

pub async fn get_playlist_music(
	State(app_state): State<AppState>,
	Query(params): Query<PlaylistQueryParams>,
) -> Response<Body> {
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(Body::from(format!("Failed to get DB connection: {}", err)))
				.unwrap();
		}
	};

	use crate::schema::{music, playlist_songs, playlists};

	// Fetch playlist details
	let playlist_result = playlists::table
		.filter(playlists::playlist_id.eq(&params.playlist_id))
		.first::<Playlist>(&mut db_conn);

	let playlist = match playlist_result {
		Ok(playlist) => playlist,
		Err(err) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(Body::from(format!("Failed to query playlist details: {}", err)))
				.unwrap();
		}
	};

	// Fetch songs in the playlist with correct type mapping
	let query_results = playlist_songs::table
		.filter(playlist_songs::playlist_id.eq(&params.playlist_id))
		.inner_join(music::table)
		.select((
			music::music_id,
			music::artist,
			music::title,
			music::album,
			music::genre,
			music::duration,
			playlist_songs::song_added_date_time,
			playlist_songs::song_adder_id,
		))
		.load::<MusicQueryResult>(&mut db_conn);

	let songs = match query_results {
		Ok(results) => results
			.into_iter()
			.map(PlaylistMusicResponse::from_query_result)
			.collect::<Vec<_>>(),
		Err(err) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(Body::from(format!("Failed to query playlist music: {}", err)))
				.unwrap();
		}
	};

	// Construct the final response
	let response = PlaylistDetailsResponse { playlist, songs };

	Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.body(Body::from(serde_json::to_string(&response).unwrap()))
		.unwrap()
}
