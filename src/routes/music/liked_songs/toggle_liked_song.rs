use crate::core::app_state::AppState; // Assuming AppState is defined in your core module
use axum::{extract::State, http::StatusCode, response::Response, Json};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// Struct for the request payload
#[derive(Debug, Serialize, Deserialize)]
pub struct ToggleLikedSong {
	pub user_id: String,
	pub music_id: String,
}

// Handler for toggling the liked state of a song
pub async fn toggle_liked_song(
	State(app_state): State<AppState>,
	Json(payload): Json<ToggleLikedSong>,
) -> Response<String> {
	// Get a database connection from the pool
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(format!("Failed to get DB from pool: {err}"))
				.unwrap();
		}
	};

	// Use the liked_songs table schema
	use crate::schema::liked_songs::dsl::*;

	// Check if the song is already liked by the user
	let is_liked = liked_songs
		.filter(user_id.eq(&payload.user_id))
		.filter(music_id.eq(&payload.music_id))
		.first::<(String, String, String)>(&mut db_conn);

	match is_liked {
		// If the song is liked, remove it
		Ok(_) => {
			match diesel::delete(liked_songs)
				.filter(user_id.eq(&payload.user_id))
				.filter(music_id.eq(&payload.music_id))
				.execute(&mut db_conn)
			{
				Ok(_) => Response::builder()
					.status(StatusCode::OK)
					.body("Song removed from liked songs".to_string())
					.unwrap(),
				Err(err) => Response::builder()
					.status(StatusCode::INTERNAL_SERVER_ERROR)
					.body(format!("Failed to remove song from liked songs: {}", err))
					.unwrap(),
			}
		}
		// If the song is not liked, add it
		Err(diesel::result::Error::NotFound) => {
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
				Err(err) => Response::builder()
					.status(StatusCode::INTERNAL_SERVER_ERROR)
					.body(format!("Failed to add song to liked songs: {}", err))
					.unwrap(),
			}
		}
		// Handle other errors
		Err(err) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Failed to toggle liked state: {}", err))
			.unwrap(),
	}
}
