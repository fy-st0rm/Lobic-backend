use crate::core::app_state::AppState;
use crate::lobic_db::models::User;
use crate::mail::mailer::send_mail;
use crate::mail::otp_mail::otp_mail;
use crate::schema::users;

use axum::{
	extract::{Path, State},
	http::status::StatusCode,
	response::Response,
	Json,
};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use rand::Rng;
use serde::Deserialize;
use std::str::FromStr;

pub async fn is_verified(State(app_state): State<AppState>, Path(user_id): Path<String>) -> Response<String> {
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

	let query = users::table
		.filter(users::user_id.eq(&user_id))
		.first::<User>(&mut db_conn);

	let user = match query {
		Ok(data) => data,
		Err(_) => {
			return Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(format!("Invalid user id: {}", &user_id))
				.unwrap();
		}
	};

	// If the verified time is not set then the user cannot be authorized
	if user.otp_verified.is_none() {
		return Response::builder()
			.status(StatusCode::UNAUTHORIZED)
			.body("OTP not verified".to_string())
			.unwrap();
	}

	let exp_time: DateTime<Utc> = DateTime::from_str(&user.otp_verified.unwrap()).unwrap();

	// Checking if otp is verified and is within the expiration limit
	if Utc::now() < exp_time {
		return Response::builder()
			.status(StatusCode::OK)
			.body("OTP verified".to_string())
			.unwrap();
	}

	// If not reseting the verification to false
	diesel::update(users::table.filter(users::user_id.eq(&user_id)))
		.set(users::otp_verified.eq::<Option<String>>(None))
		.execute(&mut db_conn)
		.unwrap();

	return Response::builder()
		.status(StatusCode::UNAUTHORIZED)
		.body("OTP not verified".to_string())
		.unwrap();
}

#[derive(Debug, Deserialize)]
pub struct VerifyOTPPayload {
	pub user_id: String,
	pub otp: String,
	pub r#for: String,
}

pub async fn verify_otp(State(app_state): State<AppState>, Json(payload): Json<VerifyOTPPayload>) -> Response<String> {
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

	let query = users::table
		.filter(users::user_id.eq(&payload.user_id))
		.first::<User>(&mut db_conn);

	let user = match query {
		Ok(data) => data,
		Err(_) => {
			return Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(format!("Invalid user id: {}", &payload.user_id))
				.unwrap();
		}
	};

	let exp_time: DateTime<Utc> = DateTime::from_str(&user.otp_expires_at).unwrap();
	if user.otp == payload.otp && Utc::now() < exp_time {
		// Making the email verified
		if &payload.r#for == "email" {
			diesel::update(users::table.filter(users::user_id.eq(&payload.user_id)))
				.set(users::email_verified.eq(true))
				.execute(&mut db_conn)
				.unwrap();
		}
		// Making the otp verified
		else if &payload.r#for == "otp" {
			let expires_at = (Utc::now() + Duration::minutes(5)).to_string();
			diesel::update(users::table.filter(users::user_id.eq(&payload.user_id)))
				.set(users::otp_verified.eq(expires_at))
				.execute(&mut db_conn)
				.unwrap();
		}

		return Response::builder()
			.status(StatusCode::OK)
			.body("OTP verified".to_string())
			.unwrap();
	}
	return Response::builder()
		.status(StatusCode::BAD_REQUEST)
		.body("Incorrect or Expired OTP".to_string())
		.unwrap();
}

pub async fn resend_otp(State(app_state): State<AppState>, Path(identifier): Path<String>) -> Response<String> {
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

	let query = if identifier.ends_with("@gmail.com") {
		users::table
			.filter(users::email.eq(&identifier))
			.first::<User>(&mut db_conn)
	} else {
		users::table
			.filter(users::user_id.eq(&identifier))
			.first::<User>(&mut db_conn)
	};

	let user = match query {
		Ok(data) => data,
		Err(_) => {
			return Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(format!("Username or Email is not registered: {}", &identifier))
				.unwrap();
		}
	};

	// Generate otp
	let mut rng = rand::rng();
	let new_otp = rng.random_range(100_000..1_000_000).to_string();
	let exp_time = (Utc::now() + Duration::minutes(5)).to_string();

	// Making the user verified

	// I know this looks stupid, cuz it is and I dont know a better way to do this. So deal with it.
	if identifier.ends_with("@gmail.com") {
		diesel::update(users::table.filter(users::email.eq(&identifier)))
			.set((users::otp.eq(new_otp.clone()), users::otp_expires_at.eq(exp_time)))
			.execute(&mut db_conn)
			.unwrap();
	} else {
		diesel::update(users::table.filter(users::user_id.eq(&identifier)))
			.set((users::otp.eq(new_otp.clone()), users::otp_expires_at.eq(exp_time)))
			.execute(&mut db_conn)
			.unwrap();
	}

	// Send the otp mail
	let mail = otp_mail(&user.email, new_otp);
	send_mail(mail);

	Response::builder()
		.status(StatusCode::OK)
		.body("Sucessfully sent a new otp".to_string())
		.unwrap()
}
