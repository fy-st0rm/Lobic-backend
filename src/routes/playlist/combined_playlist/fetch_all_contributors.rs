use crate::core::app_state::AppState;
use crate::schema::playlist_shares;
use crate::schema::playlists;
use axum::Json;
use axum::{extract::State, http::status::StatusCode, response::Response};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Contributor {
	contributor_user_id: String,
	is_writable: bool,
}

#[derive(Serialize)]
pub struct FetchContributorsResponse {
	playlist_owner: String,
	contributors: Vec<Contributor>,
}

#[derive(Deserialize)]
pub struct FetchContributorsPayload {
	playlist_id: String,
}

pub async fn fetch_all_contributors(
	State(app_state): State<AppState>,
	Json(payload): Json<FetchContributorsPayload>,
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

	// Fetch the playlist owner
	let playlist_owner: Result<String, diesel::result::Error> = playlists::table
		.filter(playlists::playlist_id.eq(&payload.playlist_id))
		.select(playlists::user_id)
		.first(&mut db_conn);

	let playlist_owner = match playlist_owner {
		Ok(owner) => owner,
		Err(err) => {
			let msg = format!("Failed to fetch playlist owner: {err}");
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(msg)
				.unwrap();
		}
	};

	// Fetch all contributors and their `is_writable` flag for the given playlist_id
	let contributors = match playlist_shares::table
		.filter(playlist_shares::playlist_id.eq(&payload.playlist_id))
		.filter(playlist_shares::contributor_user_id.is_not_null())
		.select((playlist_shares::contributor_user_id, playlist_shares::is_writable))
		.load::<(String, bool)>(&mut db_conn)
	{
		Ok(contributors) => contributors
			.into_iter()
			.map(|(contributor_user_id, is_writable)| Contributor {
				contributor_user_id,
				is_writable,
			})
			.collect(),
		Err(err) => {
			let msg = format!("Failed to fetch contributors: {err}");
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(msg)
				.unwrap();
		}
	};

	// Construct the response
	let response = FetchContributorsResponse {
		playlist_owner,
		contributors,
	};

	// Serialize the response to JSON
	let json_response = serde_json::to_string(&response).unwrap();
	Response::builder().status(StatusCode::OK).body(json_response).unwrap()
}
