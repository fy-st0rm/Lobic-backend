// models.rs
use crate::schema::*;
use diesel::{prelude::Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Insertable, Queryable, Debug)]
#[diesel(table_name = users)]
pub struct User {
	pub user_id: String,
	pub username: String,
	pub email: String,
	pub pwd_hash: String,
}

#[derive(Insertable, Queryable, Debug)]
#[diesel(table_name = user_friendship)]
pub struct UserFriendship {
	pub user_id: String,
	pub friend_id: String,
}

#[derive(Insertable, Queryable, Debug, Selectable, Serialize, Deserialize)]
#[diesel(table_name = music)]
pub struct Music {
	pub music_id: String,
	pub artist: String,
	pub title: String,
	pub album: String,
	pub genre: String,
	pub times_played: i32,
}

#[derive(Insertable, Queryable, Debug, Selectable, Serialize, Deserialize)]
#[diesel(table_name = playlists)]
pub struct Playlist {
	pub playlist_id: String,
	pub playlist_name: String,
	pub user_id: String,
	pub description: Option<String>,
	pub creation_date_time: String,
	pub last_updated_date_time: String,
}

#[derive(Insertable, Queryable, Debug)]
#[diesel(table_name = playlist_songs)]
pub struct PlaylistSong {
	pub playlist_id: String,
	pub music_id: String,
	pub song_added_date_time: String,
}

#[derive(Insertable, Queryable, Debug)]
#[diesel(table_name = playlist_shares)]
pub struct PlaylistShare {
	pub playlist_id: String,
	pub owner_user_id: String,
	pub shared_to_user_id: String,
	pub share_permission: String,
}

#[derive(Insertable, Queryable, Debug)]
#[diesel(table_name = play_log)]
pub struct PlayLog {
	pub user_id: String,
	pub music_id: String,
	pub music_played_date_time: String,
}
