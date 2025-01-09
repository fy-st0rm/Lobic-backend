use crate::core::app_state::AppState;
use crate::lobic_db::models::PlaylistSong;
use axum::{extract::State, http::status::StatusCode, response::Response, Json};
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/*
//run playlist/new first to get playlist_id in the db
post : http://127.0.0.1:8080/playlist/add_song

{
   "playlist_id":"4f537410-d5e0-4507-859b-88ecdabafd96", //must be valid
   "music_id": "b846a188-46a9-4fa4-bb7b-1b1527e7f5bd", //must be valid
}
*/

#[derive(Debug, Serialize, Deserialize)]
pub struct AddSongToPlaylist {
	pub playlist_id: String,
	pub music_id: String,
	pub position: Option<i32>,
}
//add a song to the playlist by playlist_id , music_id
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

	// Determine the position for the new song
	let pos = match payload.position {
		Some(p) => p, // Use the provided position
		None => {
			// if no postition is provided add to last
			let max_position_result = playlist_songs
				.filter(playlist_id.eq(&payload.playlist_id))
				.select(diesel::dsl::max(position))
				.first::<Option<i32>>(&mut db_conn);

			match max_position_result {
				Ok(Some(max_pos)) => max_pos + 1,
				Ok(None) => 0, // If the playlist is empty, start at position 0
				Err(err) => {
					return Response::builder()
						.status(StatusCode::INTERNAL_SERVER_ERROR)
						.body(format!("Failed to query max position: {}", err))
						.unwrap();
				}
			}
		}
	};

	let new_playlist_song = PlaylistSong {
		playlist_id: payload.playlist_id,
		music_id: payload.music_id,
		position: pos,
		song_added_date_time: curr_song_added_date_time,
	};

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
