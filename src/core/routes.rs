use crate::{
	core::app_state::AppState,
	routes::{
		auth::{
			change_password::change_password,
			login::login,
			logout::logout,
			otp::{is_verified, resend_otp, verify_otp},
			signup::signup,
			verify::{verify, verify_email},
		},
		get_lobby::get_lobby,
		music::{
			browse_category::{
				browse_albums::browse_albums, browse_artists::browse_artists, browse_genres::browse_genres,
			},
			get_cover_image::get_cover_image,
			get_music::get_music,
			liked_songs::{
				add_to_liked_song::add_to_liked_songs, get_liked_songs::get_liked_songs, is_song_liked::is_song_liked,
				remove_from_liked_songs::remove_from_liked_songs, toggle_liked_song::toggle_liked_song,
			},
			log_song_play::log_song_play,
			recently_played::get_recently_played::get_recently_played,
			save_music::save_music,
			search_music::search_music,
			send_music::send_music,
			top_tracks::get_top_tracks::get_top_tracks,
			trending::get_trending_songs::get_trending_songs,
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
		search::search,
		socket::websocket_handler,
		users::{
			add_friend::add_friend, get_friend::get_friend, get_user::get_user, get_user_data::get_user_data,
			get_user_pfp::get_user_pfp, remove_friend::remove_friend, search_user::search_user, update_pfp::update_pfp,
		},
	},
};
use axum::{
	routing::{get, post},
	Router,
};

pub fn configure_routes(app_state: AppState) -> Router {
	Router::new()
		//load musics into storage
		.route("/save_music", post(save_music))
		//auth
		.route("/", get(index))
		.route("/get_user", get(get_user))
		.route("/signup", post(signup))
		.route("/login", post(login))
		.route("/logout", post(logout))
		.route("/verify", get(verify))
		.route("/search", get(search))
		.route("/change_password", post(change_password))
		// otp
		.route("/otp/verify/:user_id", get(is_verified))
		.route("/otp/verify", post(verify_otp))
		.route("/otp/resend/:user_id", get(resend_otp))
		// email routes
		.route("/email/verify/:id", get(verify_email))
		//base
		.route("/music/:music_id", get(send_music)) //get actual mp3 music
		.route("/image/:img_uuid", get(get_cover_image)) //get the png cover image
		//music data
		.route("/search_music", get(search_music))
		.route("/music/get_music", get(get_music))
		//browse category
		.route("/music/browse_artists", get(browse_artists)) //returns Vec<artist, song_count ,Vec<image_uuid>>/ the image_uuids is capped to 4
		.route("/music/browse_albums", get(browse_albums)) //returns Vec<album, song_count ,Vec<image_uuid>>/ the image_uuids is capped to 4
		.route("/music/browse_genres", get(browse_genres)) //returns Vec<genre, song_count >
		//recently played
		.route("/music/log_song_play", post(log_song_play))
		.route("/music/get_recently_played", get(get_recently_played))
		//trending songs
		.route("/music/get_trending", get(get_trending_songs))
		//top tracks of a particular user
		.route("/music/get_top_tracks", get(get_top_tracks))
		//liked songs
		.route("/music/liked_song/add", post(add_to_liked_songs))
		.route("/music/liked_song/remove", post(remove_from_liked_songs))
		.route("/music/liked_song/get", get(get_liked_songs))
		.route("/music/liked_song/is_song_liked", get(is_song_liked))
		.route("/music/liked_song/toggle_like", post(toggle_liked_song))
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
		.route(
			"/playlist/combined/fetch_all_contributors/:playlist_id",
			get(fetch_all_contributors),
		)
		//user stuff
		.route("/user/update_pfp", post(update_pfp)) // @TODO :support non png image
		.route("/user/get_pfp/:filename", get(get_user_pfp)) // @TODO : support non png
		.route("/user/get_user_data", get(get_user_data))
		.route("/user/search", get(search_user))
		//friends stuff
		.route("/friend/add", post(add_friend))
		.route("/friend/remove", post(remove_friend))
		.route("/friend/get/:user_id", get(get_friend))
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
