use crate::config::{OpCode, SocketResponse};
use crate::core::app_state::AppState;
use crate::core::user_pool::UserPool;
use crate::lobic_db::db::DatabasePool;
use crate::lobic_db::models::{NotifModel, Notification};
use crate::schema::notifications::dsl::*;

use axum::{
	extract::{ws::Message, Path, State},
	http::status::StatusCode,
	response::Response,
};
use diesel::prelude::*;
use std::collections::HashMap;

pub fn notify(client_id: &str, notif: Notification, db_pool: &DatabasePool, user_pool: &UserPool) {
	let mut db_conn = match db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			println!("Error {}:{}: Failed to get DB from pool: {err}", file!(), line!());
			return;
		}
	};

	match user_pool.get(client_id) {
		Some(conn) => {
			// Sending to the user connection
			let response = SocketResponse {
				op_code: OpCode::NOTIFICATION,
				r#for: OpCode::NOTIFICATION,
				value: notif.clone().into(),
			}
			.to_string();
			let _ = conn.send(Message::Text(response));
		}
		None => (), // Triggered when client is offline
	};

	// Storing the notification
	diesel::insert_into(notifications)
		.values(&notif.to_model(client_id))
		.execute(&mut db_conn)
		.unwrap();
}

pub async fn get_all_notif(State(app_state): State<AppState>, Path(client_id): Path<String>) -> Response<String> {
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

	// Collecting notification with the given client id
	let query = notifications
		.filter(user_id.eq(&client_id))
		.load::<NotifModel>(&mut db_conn);

	let results = match query {
		Ok(results) => results,
		Err(_) => {
			let msg = format!("Invalid client id: {}", client_id);
			return Response::builder().status(StatusCode::BAD_REQUEST).body(msg).unwrap();
		}
	};

	// Mapping the models into the notifications
	let notifs: HashMap<String, Notification> = results
		.into_iter()
		.map(|entry| {
			let notif = Notification {
				id: entry.id.clone(),
				op_code: serde_json::from_str(&entry.op_code).unwrap(),
				value: serde_json::from_str(&entry.value).unwrap(),
			};
			(entry.id, notif)
		})
		.collect();

	// Converting to json string
	let response = serde_json::to_string(&notifs).unwrap();
	Response::builder().status(StatusCode::OK).body(response).unwrap()
}

pub async fn remove_notif(State(app_state): State<AppState>, Path(notif_id): Path<String>) -> Response<String> {
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

	// Checking if the notification with that id exists or not
	let query = notifications.filter(id.eq(&notif_id)).load::<NotifModel>(&mut db_conn);

	let _ = match query {
		Ok(_) => (),
		Err(_) => {
			let msg = format!("Invalid notification id: {}", notif_id);
			return Response::builder().status(StatusCode::BAD_REQUEST).body(msg).unwrap();
		}
	};

	// Deleting the notification
	diesel::delete(notifications.filter(id.eq(&notif_id)))
		.execute(&mut db_conn)
		.unwrap();

	Response::builder()
		.status(StatusCode::OK)
		.body("Sucessfully deleted the notification".to_string())
		.unwrap()
}
