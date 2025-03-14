use crate::utils::{cookie, exp, jwt};

use axum::{
	http::{header, status::StatusCode},
	response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use serde_json::json;

pub async fn get_user(jar: CookieJar) -> Response<String> {
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
			let response = json!({
				"user_id": claims.id
			})
			.to_string();

			return Response::builder().status(StatusCode::OK).body(response).unwrap();
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

			let access_cookie = cookie::create("access_token", &access_token, 60 * 60);
			let response = json!({
			"user_id": claims.id
			})
			.to_string();

			return Response::builder()
				.status(StatusCode::OK)
				.header(header::SET_COOKIE, access_cookie)
				.body(response)
				.unwrap();
		}
		Err(_) => (),
	};

	Response::builder()
		.status(StatusCode::UNAUTHORIZED)
		.body("Required Authentication".to_string())
		.unwrap()
}
