use crate::{
	core::app_state::AppState,
	routes::{
		auth::{login::login, logout::logout, signup::signup, verify::verify},
		get_lobby::get_lobby,
		music::{
			browse_all::browse_all,
			get_cover_image::get_cover_image,
			get_music::get_music,
			liked_songs::{
				add_to_liked_song::add_to_liked_songs, get_liked_songs::get_liked_songs, is_song_liked::is_song_liked,
				remove_from_liked_songs::remove_from_liked_songs, toggle_liked_song::toggle_liked_song,
			},
			recently_played::{get_recently_played::get_recently_played, log_song_play::log_song_play},
			save_music::save_music,
			search_music::search_music,
			send_music::send_music,
			top_tracks::get_top_tracks::get_top_tracks,
			trending::{get_trending_songs::get_trending_songs, increment_times_played::incr_times_played},
		},
		notify::{get_all_notif, remove_notif},
		playlist::{
			add_song_to_playlist::add_song_to_playlist,
			combined_playlist::{
				add_contributor::add_contributor, fetch_all_contributors::fetch_all_contributors,
				remove_contributor::remove_contributor,
			},
			create_new_playlist::create_playlist,
			delete_playlist::delete_playlist,
			get_playlist_cover_img::get_playlist_cover_img,
			get_playlist_music::get_playlist_music,
			get_users_playlists::get_users_playlists,
			remove_song_from_playlist::remove_song_from_playlist,
			update_playlist_cover_img::update_playlist_cover_img,
		},
		socket::websocket_handler,
		users::{
			add_friend::add_friend, get_user::get_user, get_user_data::get_user_data, get_user_pfp::get_user_pfp,
			remove_friend::remove_friend, search_user::search_user, update_pfp::update_pfp,
		},
	},
};
use axum::{
	routing::{get, post},
	Router,
};

pub fn configure_routes(app_state: AppState) -> Router {
	Router::new()
		//auth
		.route("/", get(index))
		.route("/get_user", get(get_user))
		.route("/signup", post(signup))
		.route("/login", post(login))
		.route("/logout", post(logout))
		.route("/verify", get(verify))
		//music
		.route("/music/:music_id", get(send_music))
		.route("/image/:id", get(get_cover_image))
		.route("/save_music", post(save_music)) // @TODO :add support for non mp3 and musci with missing tags
		.route("/music/get_music", get(get_music)) //^^^^^^^^^
		.route("/search", get(search_music))
		.route("/music/browse_all/:category", get(browse_all))
		//recently played + trending songs
		.route("/music/log_song_play", post(log_song_play))
		.route("/music/get_recently_played", get(get_recently_played))
		.route("/music/get_trending", get(get_trending_songs))
		//liked songs
		.route("/music/liked_song/add", post(add_to_liked_songs))
		.route("/music/liked_song/remove", post(remove_from_liked_songs))
		.route("/music/liked_song/get", get(get_liked_songs))
		.route("/music/liked_song/is_song_liked", get(is_song_liked))
		.route("/music/liked_song/toggle_like", post(toggle_liked_song))
		//top tracks
		.route("/music/get_top_tracks", get(get_top_tracks))
		.route("/music/incr_times_played/:music_uuid", post(incr_times_played))
		//playlist stuff
		.route("/playlist/new", post(create_playlist))
		.route("/playlist/add_song", post(add_song_to_playlist))
		.route("/playlist/get_by_uuid", get(get_playlist_music))
		.route("/playlist/get_users_playlists", get(get_users_playlists))
		.route("/playlist/update_cover_img", post(update_playlist_cover_img))
		.route("/playlist/cover_img/:playlist_id", get(get_playlist_cover_img))
		.route("/playlist/remove_song_from_playlist", post(remove_song_from_playlist))
		.route("/playlist/delete/:curr_playlist_id", post(delete_playlist))
		//combined playlists
		.route("/playlist/combined/add_contributor", post(add_contributor))
		.route("/playlist/combined/remove_contributor", post(remove_contributor))
		.route("/playlist/combined/fetch_contributors", get(fetch_all_contributors))
		//user stuff
		.route("/user/update_pfp", post(update_pfp)) // @TODO :support non png image
		.route("/user/get_pfp/:filename", get(get_user_pfp)) // @TODO : support non png
		.route("/user/get_user_data/:user_uuid", get(get_user_data))
		.route("/add_friend", post(add_friend))
		.route("/remove_friend", post(remove_friend))
		.route("/user/search", get(search_user))
		//notification
		.route("/notif/get/:client_id", get(get_all_notif))
		.route("/notif/delete/:notif_id", post(remove_notif))
		//ws
		.route("/ws", get(websocket_handler))
		.route("/get_lobby/:lobby_id", get(get_lobby))
		.with_state(app_state)
}

async fn index() -> String {
	"Hello from Lobic backend".to_string()
}
