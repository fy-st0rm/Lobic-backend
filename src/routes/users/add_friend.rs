use crate::core::app_state::AppState;
use crate::lobic_db::models::UserFriendship;
use crate::lobic_db::db::*;
use crate::schema::user_friendship::dsl::*;
use crate::config::OpCode;

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use axum::{
	extract::{State, ws::Message},
	http::status::StatusCode,
	response::Response,
	Json,
};

#[derive(Serialize, Deserialize)]
pub struct AddFriendPayload {
	pub user_id: String,
	pub friend_id: String,
}

pub async fn add_friend(
	State(app_state): State<AppState>,
	Json(payload): Json<AddFriendPayload>
) -> Response<String> {
	if !user_exists(&payload.user_id, &app_state.db_pool) {
		let msg = format!("Invalid user_id: {}", payload.user_id);
		return Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.body(msg)
			.unwrap();
	}

	if !user_exists(&payload.friend_id, &app_state.db_pool) {
		let msg = format!("Invalid friend_id: {}", payload.friend_id);
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
	
	// Querying the friendships
	let query = user_friendship
		.filter(user_id.eq(&payload.user_id))
		.load::<UserFriendship>(&mut db_conn);

	let friendships = match query {
		Ok(val) => val,
		Err(_) => {
			return Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(format!(
					"Something went wrong ({}: {})",
					file!(), line!()
				))
				.unwrap();
		}
	};

	// Checking if the intended one is already the user's friend
	for friendship in friendships {
		if friendship.friend_id == payload.friend_id {
			let msg = format!(
				"user with id: {} is already a friend of {}",
				payload.friend_id, payload.user_id
			);
			return Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(msg)
				.unwrap();
		}
	}

	// Creating a new friendship
	let new_friendship = UserFriendship {
		user_id: payload.user_id.clone(),
		friend_id: payload.friend_id.clone(),
	};

	diesel::insert_into(user_friendship)
		.values(&new_friendship)
		.execute(&mut db_conn)
		.unwrap();

	// Send notification to the friend
	let conn = match app_state.user_pool.get(&payload.friend_id) {
		Some(conn) => conn,
		None => {
			let msg = format!("Looks like user {} hasnt registered to websocket.", payload.friend_id);
			return Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(msg)
				.unwrap();
		}
	};

	let response = json!({
		"op_code": OpCode::OK,
		"for": OpCode::ADD_FRIEND,
		"value": payload.user_id
	}).to_string();
	let _ = conn.send(Message::Text(response));

	// Finish
	Response::builder()
		.status(StatusCode::OK)
		.body("Sucessfully added friend".to_string())
		.unwrap()
}

