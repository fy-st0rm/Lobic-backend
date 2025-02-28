use crate::config::OpCode;
use crate::core::app_state::AppState;
use crate::lobic_db::db::*;
use crate::lobic_db::models::{Notification, UserFriendship};
use crate::routes::notify::notify;
use crate::schema::user_friendship::dsl::*;

use axum::{extract::State, http::status::StatusCode, response::Response, Json};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AddFriendPayload {
	pub user_id: String,
	pub friend_id: String,
}

pub async fn add_friend(State(app_state): State<AppState>, Json(payload): Json<AddFriendPayload>) -> Response<String> {
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

	// Checking if the intended one is already the user's friend
	for friendship in friendships {
		if friendship.friend_id == payload.friend_id {
			let msg = format!(
				"user with id: {} is already a friend of {}",
				payload.friend_id, payload.user_id
			);
			return Response::builder().status(StatusCode::BAD_REQUEST).body(msg).unwrap();
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

	// Sending the notification only if the the targeted user is not a friend of ours (req sender)
	{
		// Querying the friendships
		let query = user_friendship
			.filter(user_id.eq(&payload.friend_id))
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

		// Checking if the intended one is already the user's friend
		let mut is_friend = false;
		for friendship in friendships {
			if friendship.friend_id == payload.user_id {
				is_friend = true;
				break;
			}
		}

		if !is_friend {
			// Send notification to the friend
			let notif = Notification::new(OpCode::ADD_FRIEND, payload.user_id.into());
			notify(&payload.friend_id, notif, &app_state.db_pool, &app_state.user_pool);
		}
	}

	// Finish
	Response::builder()
		.status(StatusCode::OK)
		.body("Sucessfully added friend".to_string())
		.unwrap()
}
