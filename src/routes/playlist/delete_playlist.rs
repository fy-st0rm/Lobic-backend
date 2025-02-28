use crate::core::app_state::AppState;
use axum::{extract::State, http::status::StatusCode, response::Response};
use diesel::prelude::*;

pub async fn delete_playlist(
	State(app_state): State<AppState>,
	axum::extract::Path(curr_playlist_id): axum::extract::Path<String>,
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

	// Use the playlists table for deletion
	use crate::schema::playlists::dsl::*;

	// Delete associated records in playlist_songs
	let songs_deleted = match diesel::delete(crate::schema::playlist_songs::dsl::playlist_songs)
		.filter(crate::schema::playlist_songs::dsl::playlist_id.eq(&curr_playlist_id))
		.execute(&mut db_conn)
	{
		Ok(count) => count,
		Err(err) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(format!("Failed to delete playlist songs: {}", err))
				.unwrap();
		}
	};

	// Delete associated records in playlist_shares
	let shares_deleted = match diesel::delete(crate::schema::playlist_shares::dsl::playlist_shares)
		.filter(crate::schema::playlist_shares::dsl::playlist_id.eq(&curr_playlist_id))
		.execute(&mut db_conn)
	{
		Ok(count) => count,
		Err(err) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(format!("Failed to delete playlist shares: {}", err))
				.unwrap();
		}
	};

	// Delete the playlist itself
	match diesel::delete(playlists)
		.filter(playlist_id.eq(&curr_playlist_id))
		.execute(&mut db_conn)
	{
		Ok(0) => Response::builder()
			.status(StatusCode::NOT_FOUND)
			.body("No playlist found to delete".to_string())
			.unwrap(),
		Ok(_) => Response::builder()
			.status(StatusCode::OK)
			.body(format!(
				"Playlist deleted. Songs deleted: {}, Shares deleted: {}",
				songs_deleted, shares_deleted
			))
			.unwrap(),
		Err(err) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("Failed to delete playlist: {}", err))
			.unwrap(),
	}
}
