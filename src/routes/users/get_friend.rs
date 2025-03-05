use crate::core::app_state::AppState;
use crate::lobic_db::db::*;
use crate::lobic_db::models::UserFriendship;
use crate::schema::user_friendship;

use serde_json::json;
use axum::{
	extract::{Path, State},
	http::status::StatusCode,
	response::Response,
};
use diesel::prelude::*;


pub async fn get_friend(State(app_state): State<AppState>, Path(user_id): Path<String>) -> Response<String> {
	if !user_exists(&user_id, &app_state.db_pool) {
		let msg = format!("Invalid user_id: {}", user_id);
		return Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.body(msg)
			.unwrap();
	}

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

	// Loading the friendship of the user
	let query = user_friendship::table
		.filter(user_friendship::user_id.eq(&user_id))
		.load::<UserFriendship>(&mut db_conn);

	let friendships = match query {
		Ok(data) => data,
		Err(_) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body("Failed to query user friendship".to_string())
				.unwrap();
		}
	};

	// Collecting all the friends ids
	let friends: Vec<String> = friendships
		.iter()
		.map(|f| f.friend_id.clone())
		.collect();

	let response = json!({
		"friends": friends
	}).to_string();
	
	Response::builder()
		.status(StatusCode::OK)
		.body(response)
		.unwrap()
}
