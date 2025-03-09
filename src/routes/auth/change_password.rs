use crate::core::app_state::AppState;
use crate::lobic_db::models::User;
use crate::schema::users;

use axum::{extract::State, http::status::StatusCode, response::Response, Json};
use diesel::prelude::*;
use pwhash::bcrypt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordPayload {
	pub user_id: String,
	pub password: String,
}

pub async fn change_password(
	State(app_state): State<AppState>,
	Json(payload): Json<ChangePasswordPayload>,
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

	let query = users::table
		.filter(users::user_id.eq(&payload.user_id))
		.first::<User>(&mut db_conn);

	match query {
		Ok(user) => user,
		Err(_) => {
			return Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(format!("Invalid User ID: {}", payload.user_id))
				.unwrap();
		}
	};

	let hash = bcrypt::hash(payload.password).unwrap();
	diesel::update(users::table.filter(users::user_id.eq(&payload.user_id)))
		.set(users::pwd_hash.eq(hash))
		.execute(&mut db_conn)
		.unwrap();

	Response::builder()
		.status(StatusCode::OK)
		.body("Sucessfully changed the password".to_string())
		.unwrap()
}
