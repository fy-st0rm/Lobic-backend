use crate::{
	app_state::AppState,
	routes::{
		auth::{login::login, signup::signup, verify::verify},
		get_user::get_user,
		music::{
			get_music::{get_cover_image, get_music, send_music},
			save_music::save_music,
			search_music::search_music,
		},
		playlist::{
			add_song_to_playlist::add_song_to_playlist, create_new_playlist::create_playlist,
			get_playlist_music::get_playlist_music,
		},
		socket::websocket_handler,
	},
};
use axum::{
	routing::{get, post},
	Router,
};

pub fn configure_routes(app_state: AppState) -> Router {
	Router::new()
		.route("/", get(index))
		.route("/get_user", get(get_user))
		.route("/signup", post(signup))
		.route("/login", post(login))
		.route("/verify", get(verify))
		.route("/music/:music_id", get(send_music))
		.route("/image/:filename", get(get_cover_image))
		.route("/save_music", post(save_music))
		.route("/get_music", get(get_music))
		.route("/search", get(search_music))
		.route("/playlist/new", post(create_playlist))
		.route("/playlist/add_song", post(add_song_to_playlist))
		.route("/playlist/get_by_uuid", get(get_playlist_music))
		.route("/ws", get(websocket_handler))
		.with_state(app_state)
}

async fn index() -> String {
	"Hello from Lobic backend".to_string()
}
