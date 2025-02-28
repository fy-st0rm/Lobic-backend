use crate::core::app_state::AppState;
use crate::lobic_db::db::*;
use crate::lobic_db::models::UserFriendship;
use crate::schema::user_friendship::dsl::*;

use axum::{extract::State, http::status::StatusCode, response::Response, Json};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RemoveFriendPayload {
	pub user_id: String,
	pub friend_id: String,
}

pub async fn remove_friend(
	State(app_state): State<AppState>,
	Json(payload): Json<RemoveFriendPayload>,
) -> Response<String> {
	if !user_exists(&payload.user_id, &app_state.db_pool) {
		let msg = format!("Invalid user_id: {}", payload.user_id);
		return Response::builder().status(StatusCode::BAD_REQUEST).body(msg).unwrap();
	}

	if !user_exists(&payload.friend_id, &app_state.db_pool) {
		let msg = format!("Invalid friend_id: {}", payload.friend_id);
		return Response::builder().status(StatusCode::BAD_REQUEST).body(msg).unwrap();
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
				.body(format!("Something went wrong ({}: {})", file!(), line!()))
				.unwrap();
		}
	};

	// Deleting the friendship from db if the relation exists
	for friendship in friendships {
		if friendship.friend_id == payload.friend_id {
			diesel::delete(user_friendship.filter(friend_id.eq(&payload.friend_id)))
				.execute(&mut db_conn)
				.unwrap();

			return Response::builder()
				.status(StatusCode::OK)
				.body("Sucessfully removed friend".to_string())
				.unwrap();
		}
	}

	// No relation found
	let msg = format!("{} is not a friend of {}", payload.friend_id, payload.user_id);
	Response::builder().status(StatusCode::BAD_REQUEST).body(msg).unwrap()
}
