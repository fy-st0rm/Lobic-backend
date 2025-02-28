use crate::core::app_state::AppState;
use crate::lobic_db::models::Playlist;
use axum::{extract::Query, extract::State, http::status::StatusCode, response::Response};
use diesel::prelude::*;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse {
	pub message: String,
}

#[derive(Debug, Serialize)]
pub struct PlaylistInfo {
	pub playlist_id: String,
	pub playlist_name: String,
	pub creation_date_time: String,
	pub last_updated_date_time: String,
	pub is_playlist_combined: bool,
}

#[derive(Debug, Serialize)]
pub struct UserPlaylistsResponse {
	pub user_id: String,
	pub playlists: Vec<PlaylistInfo>,
}

#[derive(Debug, Deserialize)]
pub struct UserPlaylistsQuery {
	pub user_uuid: String,
}

pub async fn get_users_playlists(
	State(app_state): State<AppState>,
	Query(query): Query<UserPlaylistsQuery>,
) -> Response<String> {
	let user_uuid = query.user_uuid; // Extract user_uuid from query parameters

	let mut db_conn = match app_state.db_pool.get() {
		Ok(conn) => conn,
		Err(err) => {
			let response = ApiResponse {
				message: format!("Failed to get DB from pool: {err}"),
			};
			return Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(serde_json::to_string(&response).unwrap())
				.unwrap();
		}
	};

	use crate::schema::playlists::dsl::*;

	// Query all playlists for the given user_id
	let result = playlists.filter(user_id.eq(&user_uuid)).load::<Playlist>(&mut db_conn);

	match result {
		Ok(user_playlists) => {
			if user_playlists.is_empty() {
				let response = ApiResponse {
					message: "No playlists found for this user".to_string(),
				};
				Response::builder()
					.status(StatusCode::NOT_FOUND)
					.body(serde_json::to_string(&response).unwrap())
					.unwrap()
			} else {
				// Map the Playlist objects to PlaylistInfo
				let playlists_info: Vec<PlaylistInfo> = user_playlists
					.into_iter()
					.map(|playlist| PlaylistInfo {
						playlist_id: playlist.playlist_id,
						playlist_name: playlist.playlist_name,
						creation_date_time: playlist.creation_date_time,
						last_updated_date_time: playlist.last_updated_date_time,
						is_playlist_combined: playlist.is_playlist_combined,
					})
					.collect();

				let response = UserPlaylistsResponse {
					user_id: user_uuid,
					playlists: playlists_info,
				};
				Response::builder()
					.status(StatusCode::OK)
					.body(serde_json::to_string(&response).unwrap())
					.unwrap()
			}
		}
		Err(err) => {
			let response = ApiResponse {
				message: format!("Failed to query database: {}", err),
			};
			Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(serde_json::to_string(&response).unwrap())
				.unwrap()
		}
	}
}
