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
pub struct RecentlyPlayedResponse {
	pub music_id: String,
	pub artist: String,
	pub title: String,
	pub album: String,
	pub genre: String,
	pub music_played_date_time: String,
}

#[derive(Debug, Deserialize)]
pub struct RecentlyPlayedQueryParams {
	pub user_id: String,
	pub pagination_limit: i64,
}

pub async fn get_recently_played(
	State(app_state): State<AppState>,
	Query(params): Query<RecentlyPlayedQueryParams>,
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

	use crate::schema::{music, play_log};

	// Fetch recently played songs for the user
	let songs_query = play_log::table
		.filter(play_log::user_id.eq(&params.user_id))
		.order(play_log::music_played_date_time.desc()) // Most recent first
		.limit(params.pagination_limit) // Limit the number of results
		.inner_join(music::table)
		.select((
			music::music_id,
			music::artist,
			music::title,
			music::album,
			music::genre,
			play_log::music_played_date_time,
		))
		.into_boxed();

	let songs = match songs_query.load::<RecentlyPlayedResponse>(&mut db_conn) {
		Ok(songs) => songs,
		Err(err) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(Body::from(format!("Failed to query recently played songs: {}", err)))
				.unwrap();
		}
	};

	// Construct the final response
	Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "application/json")
		.body(Body::from(serde_json::to_string(&songs).unwrap()))
		.unwrap()
}
