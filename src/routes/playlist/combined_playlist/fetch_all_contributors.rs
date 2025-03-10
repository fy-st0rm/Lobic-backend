use crate::core::app_state::AppState;
use crate::schema::playlist_shares;
use crate::schema::playlists;
use axum::extract::Path;
use axum::{extract::State, http::status::StatusCode, response::Response};
use diesel::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct Contributor {
	contributor_user_id: String,
}

#[derive(Serialize)]
pub struct FetchContributorsResponse {
	playlist_owner: String,
	contributors: Vec<Contributor>,
}

pub async fn fetch_all_contributors(
	State(app_state): State<AppState>,
	Path(playlist_id): Path<String>,
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
		.filter(playlists::playlist_id.eq(&playlist_id))
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

	let contributors = match playlist_shares::table
		.filter(playlist_shares::playlist_id.eq(&playlist_id))
		.select(playlist_shares::contributor_user_id)
		.load::<String>(&mut db_conn)
	{
		Ok(contributors) => contributors
			.into_iter()
			.map(|contributor_user_id| Contributor { contributor_user_id })
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
