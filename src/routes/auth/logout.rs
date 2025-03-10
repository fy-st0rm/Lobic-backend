use crate::core::app_state::AppState;
use crate::utils::cookie;

use axum::{
	extract::State,
	http::{header, status::StatusCode},
	response::Response,
	Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutPayload {
	pub user_id: String,
}

pub async fn logout(State(app_state): State<AppState>, Json(payload): Json<LogoutPayload>) -> Response<String> {
	let _ = app_state.user_pool.remove(&payload.user_id);

	let user_cookie = cookie::create("user_id", "", 0);
	let access_cookie = cookie::create("access_token", "", 0);
	let refresh_cookie = cookie::create("refresh_token", "", 0);

	Response::builder()
		.status(StatusCode::OK)
		.header(header::SET_COOKIE, user_cookie)
		.header(header::SET_COOKIE, access_cookie)
		.header(header::SET_COOKIE, refresh_cookie)
		.body("Logout sucessfull".to_string())
		.unwrap()
}
