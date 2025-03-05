use crate::core::app_state::AppState;
use crate::lobic_db::models::User;
use crate::schema::users::dsl::*;

use axum::{
	extract::{Query, State},
	http::StatusCode,
	response::Response,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchUserResponse {
	pub id: String,
	pub username: String,
	pub email: String,
	pub pfp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchUserQuery {
	pub search_string: String,
	pub max_results: i64,
}

pub async fn search_user(State(app_state): State<AppState>, Query(params): Query<SearchUserQuery>) -> Response<String> {
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

	// Searching in db
	let search_query = format!("%{}%", params.search_string.to_lowercase());
	let query = users
		.filter(username.like(&search_query).or(email.like(&search_query)))
		.limit(params.max_results)
		.load::<User>(&mut db_conn);

	let matches = match query {
		Ok(matches) => matches,
		Err(err) => {
			let msg = format!("Failed to search users: {}", err);
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(msg)
				.unwrap();
		}
	};

	// Mapping the results into a reponse structure
	let results: Vec<SearchUserResponse> = matches
		.into_iter()
		.map(|entry| {
			SearchUserResponse {
				id: entry.user_id.clone(),
				username: entry.username,
				email: entry.email,
				pfp: entry.user_id,
			}
		})
		.collect::<Vec<_>>();

	// Converting to json and returning the result
	let response = json!({
		"results": results
	})
	.to_string();
	Response::builder().status(StatusCode::OK).body(response).unwrap()
}
