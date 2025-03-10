use crate::core::app_state::AppState;
use crate::lobic_db::models::User;
use crate::schema::users;
use crate::utils::{cookie, exp, jwt};

use axum::{
	extract::{Path, State},
	http::{header, status::StatusCode},
	response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use diesel::prelude::*;

pub async fn verify(jar: CookieJar) -> Response<String> {
	let access_token = match jar.get("access_token") {
		Some(token) => token,
		None => {
			return Response::builder()
				.status(StatusCode::UNAUTHORIZED)
				.body("No access token provided".to_string())
				.unwrap();
		}
	};

	let refresh_token = match jar.get("refresh_token") {
		Some(token) => token,
		None => {
			return Response::builder()
				.status(StatusCode::UNAUTHORIZED)
				.body("No refresh token provided".to_string())
				.unwrap();
		}
	};

	let secret_key = std::env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set in .env file");

	// Verifying the access token
	match jwt::verify(access_token.value(), &secret_key) {
		Ok(data) => {
			let claims = data.claims;
			let user_cookie = cookie::create("user_id", &claims.id, 60 * 60);
			return Response::builder()
				.status(StatusCode::OK)
				.header(header::SET_COOKIE, user_cookie)
				.body("OK".to_string())
				.unwrap();
		}
		Err(_) => (),
	};

	// Verifying the refresh token
	match jwt::verify(refresh_token.value(), &secret_key) {
		Ok(data) => {
			let claims = data.claims;

			// Generating new access token
			let access_claims = jwt::Claims {
				id: claims.id.clone(),
				exp: exp::expiration_from_sec(10),
			};
			let access_token = match jwt::generate(access_claims, &secret_key) {
				Ok(token) => token,
				Err(err) => {
					return Response::builder()
						.status(StatusCode::INTERNAL_SERVER_ERROR)
						.body(err.to_string())
						.unwrap();
				}
			};

			let user_cookie = cookie::create("user_id", &claims.id, 60 * 60);
			let access_cookie = cookie::create("access_token", &access_token, 60 * 60);
			return Response::builder()
				.status(StatusCode::OK)
				.header(header::SET_COOKIE, user_cookie)
				.header(header::SET_COOKIE, access_cookie)
				.body("OK".to_string())
				.unwrap();
		}
		Err(_) => (),
	};

	Response::builder()
		.status(StatusCode::UNAUTHORIZED)
		.body("Required Authentication".to_string())
		.unwrap()
}

pub async fn verify_email(State(app_state): State<AppState>, Path(id): Path<String>) -> Response<String> {
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			let msg = format!("Error {}:{}: Failed to get DB from pool: {err}", file!(), line!());
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(msg)
				.unwrap();
		}
	};

	let res = users::table.filter(users::user_id.eq(id))
		.first::<User>(&mut db_conn)
		.unwrap()
		.email_verified;

	if res {
		return Response::builder()
			.status(StatusCode::OK)
			.body("Email verified".to_string())
			.unwrap();
	}
	return Response::builder()
		.status(StatusCode::UNAUTHORIZED)
		.body("Email not verified".to_string())
		.unwrap();
}
