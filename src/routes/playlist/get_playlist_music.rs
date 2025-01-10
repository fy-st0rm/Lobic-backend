use crate::core::app_state::AppState;
use axum::{
	body::Body,
	extract::{Query, State},
	http::{header, status::StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct PlaylistMusicResponse {
	pub music_id: String,
	pub artist: String,
	pub title: String,
	pub album: String,
	pub genre: String,
	pub position: i32,
	pub song_added_date_time: String,
}

#[derive(Debug, Deserialize, Queryable)]
pub struct PlaylistQueryParams {
	pub playlist_id: String,
}

use crate::lobic_db::models::Playlist;
#[derive(Debug, Serialize)]
pub struct PlaylistDetailsResponse {
	pub playlist: Playlist,
	pub songs: Vec<PlaylistMusicResponse>,
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

	// Fetch songs in the playlist
	let songs_query = playlist_songs::table
		.filter(playlist_songs::playlist_id.eq(&params.playlist_id))
		.inner_join(music::table)
		.select((
			music::music_id,
			music::artist,
			music::title,
			music::album,
			music::genre,
			playlist_songs::position,
			playlist_songs::song_added_date_time,
		))
		.into_boxed();

	let songs = match songs_query.load::<PlaylistMusicResponse>(&mut db_conn) {
		Ok(songs) => songs,
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
