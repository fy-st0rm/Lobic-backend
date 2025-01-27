pub mod music {
	pub mod browse_all;
	pub mod get_cover_image;
	pub mod get_music;
	pub mod save_music;
	pub mod search_music;
	pub mod send_music;
	pub mod recently_played {
		pub mod get_recently_played;
		pub mod log_song_play;
	}
	pub mod trending {
		pub mod get_trending_songs;
		pub mod increment_times_played;
	}
	pub mod top_tracks {
		pub mod get_top_tracks;
	}
	pub mod liked_songs {
		pub mod add_to_liked_song;
		pub mod get_liked_songs;
		pub mod is_song_liked;
		pub mod remove_from_liked_songs;
		pub mod toggle_liked_song;
	}
}
pub mod playlist {
	pub mod add_song_to_playlist;
	pub mod create_new_playlist;
	pub mod delete_playlist;
	pub mod get_playlist_cover_img;
	pub mod get_playlist_music;
	pub mod get_users_playlists;
	pub mod remove_song_from_playlist;
	pub mod update_playlist_cover_img;
	pub mod combined_playlist {
		pub mod add_contributor;
		pub mod fetch_all_contributors;
		pub mod remove_contributor;
	}
}
pub mod users {
	pub mod add_friend;
	pub mod get_user;
	pub mod get_user_data;
	pub mod get_user_pfp;
	pub mod remove_friend;
	pub mod search_user;
	pub mod update_pfp;
}

pub mod auth {
	pub mod login;
	pub mod logout;
	pub mod signup;
	pub mod verify;
}
pub mod get_lobby;
pub mod notify;
pub mod socket;
