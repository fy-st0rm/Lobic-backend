use crate::core::app_state::AppState;
use crate::schema::playlist_shares;
use axum::Json;
use axum::{extract::State, http::status::StatusCode, response::Response};
use diesel::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RemoveContributorPayload {
	playlist_id: String,
	contributor_user_id: String,
}

pub async fn remove_contributor(
	State(app_state): State<AppState>,
	Json(payload): Json<RemoveContributorPayload>,
) -> Response<String> {
	// Attempt to get a database connection from the pool
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

	// Attempt to delete the contributor from the playlist_shares table
	match diesel::delete(
		playlist_shares::table.filter(
			playlist_shares::playlist_id
				.eq(payload.playlist_id)
				.and(playlist_shares::contributor_user_id.eq(payload.contributor_user_id)),
		),
	)
	.execute(&mut db_conn)
	{
		Ok(0) => {
			// No rows were affected, meaning the contributor was not found
			Response::builder()
				.status(StatusCode::NOT_FOUND)
				.body("Contributor not found".to_string())
				.unwrap()
		}
		Ok(_) => {
			// Contributor was successfully removed
			Response::builder()
				.status(StatusCode::OK)
				.body("Successfully removed contributor".to_string())
				.unwrap()
		}
		Err(err) => {
			// An error occurred while trying to remove the contributor
			let msg = format!("Failed to remove contributor: {err}");
			Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(msg)
				.unwrap()
		}
	}
}
