use crate::{core::app_state::AppState, schema::music::dsl::*};
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::IntoResponse,
};
use diesel::{result::Error as DieselError, Connection, RunQueryDsl};
use diesel::{update, ExpressionMethods, QueryDsl};

pub async fn incr_times_played(Path(music_uuid): Path<String>, State(app_state): State<AppState>) -> impl IntoResponse {
	// Validate music_uuid format
	if !is_valid_music_id(&music_uuid) {
		return (StatusCode::BAD_REQUEST, "Invalid music UUID format").into_response();
	}

	// Get database connection from pool
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			let msg = format!("Database connection error: {}", err);
			return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response();
		}
	};

	// Update play count with retry logic
	const MAX_RETRIES: u32 = 3;
	let mut retries = 0;
	let update_result = loop {
		match db_conn.transaction(|conn| {
			update(music.filter(music_id.eq(&music_uuid)))
				.set(times_played.eq(times_played + 1))
				.execute(conn)
		}) {
			Ok(result) => break Ok(result),
			Err(DieselError::DatabaseError(diesel::result::DatabaseErrorKind::Unknown, _)) if retries < MAX_RETRIES => {
				retries += 1;
				tokio::time::sleep(tokio::time::Duration::from_millis(10 * retries as u64)).await;
				continue;
			}
			Err(err) => break Err(err),
		}
	};

	match update_result {
		Ok(_) => (StatusCode::OK, "Play count incremented successfully").into_response(),
		Err(err) => {
			let msg = format!("Database update error: {}", err);
			(StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
		}
	}
}

// Helper function to validate music_id format
fn is_valid_music_id(id: &str) -> bool {
	!id.is_empty() && id.len() < 100 && id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}
