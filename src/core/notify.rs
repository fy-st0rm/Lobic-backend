use crate::config::{OpCode, SocketResponse};
use crate::core::user_pool::UserPool;

use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
	pub id: String,
	pub op_code: OpCode,
	pub value: Value,
}

impl Notification {
	pub fn new(op_code: OpCode, value: Value) -> Self {
		let id = Uuid::new_v4().to_string();
		Notification {
			id: id,
			op_code: op_code,
			value: value,
		}
	}
}

impl From<Notification> for Value {
	fn from(notif: Notification) -> Self {
		serde_json::to_value(&notif).unwrap()
	}
}


pub fn notify(user_id: &str, notif: Notification, user_pool: &UserPool) {
	let conn = match user_pool.get(user_id) {
		Some(conn) => conn,
		None => {
			println!("ERROR {}:{}: Looks like user {} hasnt registered to websocket.", file!(), line!(), user_id);
			return;
		}
	};

	let response = SocketResponse {
		op_code: OpCode::NOTIFICATION,
		r#for: OpCode::NOTIFICATION,
		value: notif.into()
	}.to_string();
	let _ = conn.send(Message::Text(response));
}
