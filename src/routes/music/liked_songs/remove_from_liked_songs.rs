use crate::core::app_state::AppState;
use crate::lobic_db::models::LikedSongs;

use axum::{extract::State, http::status::StatusCode, response::Response, Json};
use diesel::prelude::*;

pub async fn remove_from_liked_songs(
	State(app_state): State<AppState>,
	Json(payload): Json<LikedSongs>,
) -> Response<String> {
	// Get a database connection
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(format!("Failed to get DB from pool: {err}"))
				.unwrap();
		}
	};

	use crate::schema::liked_songs::dsl::*;

	//Delete the record from the liked_songs table
	match diesel::delete(liked_songs)
		.filter(user_id.eq(&payload.user_id))
		.filter(music_id.eq(&payload.music_id))
		.execute(&mut db_conn)
	{
		Ok(rows_deleted) => {
			if rows_deleted > 0 {
				// If a record was deleted
				Response::builder()
					.status(StatusCode::OK)
					.body("Song removed from liked songs".to_string())
					.unwrap()
			} else {
				// If no record was found to delete
				Response::builder()
					.status(StatusCode::NOT_FOUND)
					.body("Song not found in liked songs".to_string())
					.unwrap()
			}
		}
		Err(err) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Failed to remove song from liked songs: {}", err))
			.unwrap(),
	}
}
