use crate::core::app_state::AppState;
use axum::{extract::State, http::status::StatusCode, response::Response, Json};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddLikedSong {
	pub user_id: String,
	pub music_id: String,
}

pub async fn add_to_liked_songs(
	State(app_state): State<AppState>,
	Json(payload): Json<AddLikedSong>,
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
	let curr_song_added_date_time = Utc::now().to_rfc3339();

	// Create a new LikedSong record
	let new_liked_song = (
		user_id.eq(&payload.user_id),
		music_id.eq(&payload.music_id),
		song_added_date_time.eq(&curr_song_added_date_time),
	);

	// Insert the new liked song into the database
	match diesel::insert_into(liked_songs)
		.values(&new_liked_song)
		.execute(&mut db_conn)
	{
		Ok(_) => Response::builder()
			.status(StatusCode::CREATED)
			.body("Song added to liked songs".to_string())
			.unwrap(),
		Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _)) => {
			Response::builder()
				.status(StatusCode::CONFLICT)
				.body("Song already exists in liked songs".to_string())
				.unwrap()
		}
		Err(err) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Failed to add song to liked songs: {}", err))
			.unwrap(),
	}
}
