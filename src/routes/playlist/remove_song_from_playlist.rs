use crate::core::app_state::AppState;
use crate::schema::playlist_songs::dsl::*;
use axum::{extract::State, http::status::StatusCode, response::Response, Json};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveSongFromPlaylist {
	pub playlist_id: String,
	pub music_id: String,
}

pub async fn remove_song_from_playlist(
	State(app_state): State<AppState>,
	Json(payload): Json<RemoveSongFromPlaylist>,
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

	match diesel::delete(playlist_songs)
		.filter(music_id.eq(&payload.music_id))
		.filter(playlist_id.eq(&payload.playlist_id))
		.execute(&mut db_conn)
	{
		Ok(rows_deleted) => {
			if rows_deleted > 0 {
				// If a record was deleted
				Response::builder()
					.status(StatusCode::OK)
					.body(format!(
						"song {} removed from playlist {}",
						payload.music_id, payload.playlist_id
					))
					.unwrap()
			} else {
				// If no record was found to delete
				Response::builder()
					.status(StatusCode::NOT_FOUND)
					.body(format!(
						"song {} NOT FOUND playlist {}",
						payload.music_id, payload.playlist_id
					))
					.unwrap()
			}
		}
		Err(err) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Failed to remove song from playlist: {}", err))
			.unwrap(),
	}
}
