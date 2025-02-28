use crate::core::app_state::AppState;
use axum::{
	extract::{Query, State},
	http::StatusCode,
	response::Response,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckLikedSongParams {
	pub user_id: String,
	pub music_id: String,
}

pub async fn is_song_liked(
	State(app_state): State<AppState>,
	Query(payload): Query<CheckLikedSongParams>,
) -> Response<String> {
	// Get a database connection
	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(format!("{err}"))
				.unwrap();
		}
	};

	use crate::schema::liked_songs::dsl::*;

	// Check if the song is liked by the user
	let is_liked = liked_songs
		.filter(user_id.eq(&payload.user_id))
		.filter(music_id.eq(&payload.music_id))
		.first::<(String, String, String)>(&mut db_conn);

	match is_liked {
		Ok(_) => Response::builder()
			.status(StatusCode::OK)
			.body(format!("true"))
			.unwrap(),
		Err(diesel::result::Error::NotFound) => Response::builder()
			.status(StatusCode::OK)
			.body(format!("false"))
			.unwrap(),
		Err(err) => Response::builder()
			.status(StatusCode::INTERNAL_SERVER_ERROR)
			.body(format!("{err}"))
			.unwrap(),
	}
}
