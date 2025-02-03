use crate::core::app_state::AppState;
use axum::{
	extract::State,
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct GenreResult {
	genre: String,
	song_count: i64,
}

pub async fn browse_genres(State(app_state): State<AppState>) -> Response<String> {
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
		.group_by(genre)
		.select((genre, diesel::dsl::count(music_id)))
		.load::<(String, i64)>(&mut db_conn);

	match result {
		Ok(items) => {
			let category_results: Vec<GenreResult> = items
				.into_iter()
				.map(|(_genre, song_count)| GenreResult {
					genre: _genre,
					song_count,
				})
				.collect();

			let response = serde_json::to_string(&category_results).unwrap();
			Response::builder()
				.status(StatusCode::OK)
				.header(header::CONTENT_TYPE, "application/json")
				.body(response)
				.unwrap()
		}
		Err(err) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Database error: {err}"))
			.unwrap(),
	}
}
