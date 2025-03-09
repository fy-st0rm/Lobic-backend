use crate::core::app_state::AppState;
use crate::lobic_db::models::User;
use crate::schema::users;

use axum::{
	extract::{Query, State},
	http::status::StatusCode,
	response::Response,
};
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct GetUserDataQuery {
	pub user_id: Option<String>,
	pub email: Option<String>,
}

pub async fn get_user_data(
	State(app_state): State<AppState>,
	Query(params): Query<GetUserDataQuery>,
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

	// Query the users table for the user with the given user_uuid
	let query = if let Some(user_id) = params.user_id {
		users::table
			.filter(users::user_id.eq(&user_id))
			.first::<User>(&mut db_conn)
	} else if let Some(email) = params.email {
		users::table.filter(users::email.eq(&email)).first::<User>(&mut db_conn)
	} else {
		return Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.body("Query is empty".to_string())
			.unwrap();
	};

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
			.status(StatusCode::BAD_REQUEST)
			.body(format!("No user found: {err}"))
			.unwrap(),
	}
}
