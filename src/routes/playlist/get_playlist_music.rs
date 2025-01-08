use crate::app_state::AppState;
use axum::{
	body::Body,
	extract::{Query, State},
	http::{header, status::StatusCode},
	response::Response,
};
use diesel::prelude::*;

use serde::{Deserialize, Serialize};
//get : http://127.0.0.1:8080/playlist/get_by_uuid/?playlist_id=71780897-79bf-488d-9e82-4aaad3561986
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

//get all songs in the playlist by playlist_id
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

	use crate::schema::{music, playlist_songs};

	let query = playlist_songs::table
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

	match query.load::<PlaylistMusicResponse>(&mut db_conn) {
		Ok(music_list) => Response::builder()
			.status(StatusCode::OK)
			.header(header::CONTENT_TYPE, "application/json")
			.body(Body::from(serde_json::to_string(&music_list).unwrap()))
			.unwrap(),
		Err(err) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(Body::from(format!("Failed to query playlist music: {}", err)))
			.unwrap(),
	}
}
