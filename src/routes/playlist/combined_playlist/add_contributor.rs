use crate::core::app_state::AppState;
use crate::lobic_db::models::PlaylistShare;
use crate::schema::{playlist_shares, playlists};
use axum::Json;
use axum::{extract::State, http::status::StatusCode, response::Response};
use diesel::prelude::*;

pub async fn add_contributor(
	State(app_state): State<AppState>,
	Json(payload): Json<PlaylistShare>,
) -> Response<String> {
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

	// Check if playlist is combined
	let is_combined = match playlists::table
		.select(playlists::is_playlist_combined)
		.filter(playlists::playlist_id.eq(&payload.playlist_id))
		.first::<bool>(&mut db_conn)
	{
		Ok(combined) => combined,
		Err(err) => {
			let msg = format!("Failed to check playlist status: {err}");
			return Response::builder().status(StatusCode::NOT_FOUND).body(msg).unwrap();
		}
	};

	if !is_combined {
		return Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.body("Cannot add contributors to a solo playlist".to_string())
			.unwrap();
	}

	match diesel::insert_into(playlist_shares::table)
		.values(&payload)
		.execute(&mut db_conn)
	{
		Ok(_) => Response::builder()
			.status(StatusCode::OK)
			.body("Successfully added or updated contributor".to_string())
			.unwrap(),
		Err(err) => {
			let msg = format!("Failed to add/update contributor: {err}");
			Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(msg)
				.unwrap()
		}
	}
}
