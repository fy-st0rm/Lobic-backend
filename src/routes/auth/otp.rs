use crate::core::app_state::AppState;
use crate::lobic_db::models::User;
use crate::schema::users;
use crate::mail::otp_mail::otp_mail;
use crate::mail::mailer::send_mail;

use chrono::{Utc, DateTime, Duration};
use axum::{
	extract::{State, Path, Query},
	http::status::StatusCode,
	response::Response,
};
use diesel::prelude::*;
use serde::Deserialize;
use std::str::FromStr;
use rand::Rng;

#[derive(Debug, Deserialize)]
pub struct VerifyOTPQuery {
	pub user_id: String,
	pub otp: String,
}

pub async fn verify_otp(State(app_state): State<AppState>, Query(params): Query<VerifyOTPQuery>) -> Response<String> {
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
		.filter(users::user_id.eq(&params.user_id))
		.first::<User>(&mut db_conn);

	let user = match query {
		Ok(data) => data,
		Err(_) => {
			return Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(format!("Invalid user id: {}", &params.user_id))
				.unwrap();
		}
	};

	let exp_time: DateTime<Utc> = DateTime::from_str(&user.otp_expires_at).unwrap();
	if user.otp == params.otp && Utc::now() < exp_time {
		// Making the user verified
		diesel::update(users::table.filter(users::user_id.eq(&params.user_id)))
			.set(users::email_verified.eq(true))
			.execute(&mut db_conn)
			.unwrap();

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

pub async fn resend_otp(State(app_state): State<AppState>, Path(user_id): Path<String>) -> Response<String> {
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

	// Generate otp
	let mut rng = rand::rng();
	let new_otp = rng.random_range(100_000..1_000_000).to_string();
	let exp_time = (Utc::now() + Duration::minutes(5)).to_string();

	// Making the user verified
	diesel::update(users::table.filter(users::user_id.eq(&user_id)))
		.set((
			users::otp.eq(new_otp.clone()),
			users::otp_expires_at.eq(exp_time)
		))
		.execute(&mut db_conn)
		.unwrap();

	// Send the otp mail
	let mail = otp_mail(&user.email, new_otp);
	send_mail(mail);

	Response::builder()
		.status(StatusCode::OK)
		.body("Sucessfully sent a new otp".to_string())
		.unwrap()
}
