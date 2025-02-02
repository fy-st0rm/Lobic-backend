use crate::core::app_state::AppState;
use axum::{
	extract::{Path, State},
	http::{header, StatusCode},
	response::Response,
};
use diesel::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct CategoryResult {
	name: String,
	song_count: i64,
}

pub async fn browse_all(State(app_state): State<AppState>, Path(category): Path<String>) -> Response<String> {
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

	let result = match category.as_str() {
		"artists" => music
			.group_by(artist)
			.select((artist, diesel::dsl::count(music_id)))
			.load::<(String, i64)>(&mut db_conn),
		"albums" => music
			.group_by(album)
			.select((album, diesel::dsl::count(music_id)))
			.load::<(String, i64)>(&mut db_conn),
		"genres" => music
			.group_by(genre)
			.select((genre, diesel::dsl::count(music_id)))
			.load::<(String, i64)>(&mut db_conn),
		_ => {
			return Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body("Invalid category. Use 'artists', 'albums', or 'genres'.".to_string())
				.unwrap();
		}
	};

	match result {
		Ok(items) => {
			let category_results: Vec<CategoryResult> = items
				.into_iter()
				.map(|(name, song_count)| CategoryResult { name, song_count })
				.collect();

			let response = format!(
				r#"{{"category": "{}", "result": {}}}"#,
				category,
				serde_json::to_string(&category_results).unwrap()
			);
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
