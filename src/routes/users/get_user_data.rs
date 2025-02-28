use crate::core::app_state::AppState;
use crate::lobic_db::db::*;
use crate::lobic_db::models::User;
use crate::schema::users::dsl::*;

use axum::{
	extract::{Path, State},
	http::status::StatusCode,
	response::Response,
};
use diesel::prelude::*;
use serde_json::json;

pub async fn get_user_data(
	State(app_state): State<AppState>,
	Path(user_uuid): Path<String>, // Extract user_uuid from the path
) -> Response<String> {
	// Check if the user exists
	if !user_exists(&user_uuid, &app_state.db_pool) {
		let msg = format!("Invalid user_id: {}", user_uuid);
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

	// Query the users table for the user with the given user_uuid
	let query = users.filter(user_id.eq(&user_uuid)).first::<User>(&mut db_conn);

	match query {
		Ok(user) => {
			let user_data = json!({
				"id": user.user_id.clone(),
				"username": user.username,
				"email": user.email,
			})
			.to_string();
			Response::builder().status(StatusCode::OK).body(user_data).unwrap()
		}
		Err(err) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("no user found;{err}").to_string())
			.unwrap(),
	}
}
