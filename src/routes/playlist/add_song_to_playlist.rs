use crate::core::app_state::AppState;
use crate::lobic_db::models::PlaylistSong;
use axum::{extract::State, http::status::StatusCode, response::Response, Json};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddSongToPlaylist {
	pub playlist_id: String,
	pub music_id: String,
	pub song_adder_id: String,
}

pub async fn add_song_to_playlist(
	State(app_state): State<AppState>,
	Json(payload): Json<AddSongToPlaylist>,
) -> Response<String> {
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(format!("Failed to get DB from pool: {err}"))
				.unwrap();
		}
	};

	use crate::schema::playlist_songs::dsl::*;
	let curr_song_added_date_time = Utc::now().to_rfc3339();

	// Create a new PlaylistSong record
	let new_playlist_song = PlaylistSong {
		playlist_id: payload.playlist_id,
		music_id: payload.music_id,
		song_added_date_time: curr_song_added_date_time,
		song_adder_id: payload.song_adder_id,
	};

	// Insert the new song into the playlist
	match diesel::insert_into(playlist_songs)
		.values(&new_playlist_song)
		.execute(&mut db_conn)
	{
		Ok(_) => Response::builder()
			.status(StatusCode::CREATED)
			.body("Song added to playlist".to_string())
			.unwrap(),
		Err(err) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Failed to add song to playlist: {}", err))
			.unwrap(),
	}
}
