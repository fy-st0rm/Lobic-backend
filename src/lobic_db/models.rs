use std::hash::{DefaultHasher, Hash, Hasher};

use crate::config::OpCode;
use crate::schema::*;

use diesel::{prelude::Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Insertable, Queryable, Debug, Serialize, Deserialize, Selectable)]
#[diesel(table_name = users)]
pub struct User {
	pub user_id: String,
	pub username: String,
	pub email: String,
	pub pwd_hash: String,
	pub email_verified: bool,
	pub otp: String,
	pub otp_expires_at: String,
	pub otp_verified: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserDataResponse {
	pub user_id: String,
	pub username: String,
	pub email: String,
}

#[derive(Insertable, Queryable, Debug)]
#[diesel(table_name = user_friendship)]
pub struct UserFriendship {
	pub user_id: String,
	pub friend_id: String,
}

#[derive(Insertable, Queryable, Debug, Selectable, Serialize, Deserialize)]
#[diesel(table_name = playlists)]
pub struct Playlist {
	pub playlist_id: String,
	pub playlist_name: String,
	pub user_id: String,
	pub creation_date_time: String,
	pub last_updated_date_time: String,
	pub is_playlist_combined: bool,
}
//for response
#[derive(Debug, Serialize)]
pub struct PlaylistInfo {
	pub playlist_id: String,
	pub user_id: String,
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

#[derive(Insertable, Queryable, Debug)]
#[diesel(table_name = playlist_songs)]
pub struct PlaylistSong {
	pub playlist_id: String,
	pub music_id: String,
	pub song_adder_id: String,
	pub song_added_date_time: String,
}

#[derive(Insertable, Queryable, Debug, Selectable, Serialize, Deserialize)]
#[diesel(table_name = playlist_shares)]
pub struct PlaylistShare {
	pub playlist_id: String,
	pub contributor_user_id: String,
}

#[derive(Insertable, Queryable, Debug)]
#[diesel(table_name = play_log)]
pub struct PlayLog {
	pub user_id: String,
	pub music_id: String,
	pub music_played_date_time: String,
	pub user_times_played: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LikedSongs {
	pub user_id: String,
	pub music_id: String,
	pub song_added_date_time: String,
}

#[derive(Insertable, Queryable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = notifications)]
pub struct NotifModel {
	pub id: String,
	pub user_id: String,
	pub op_code: String,
	pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

	pub fn to_model(&self, user_id: &str) -> NotifModel {
		NotifModel {
			id: self.id.clone(),
			user_id: user_id.to_string(),
			op_code: serde_json::to_string(&self.op_code).unwrap(),
			value: serde_json::to_string(&self.value).unwrap(),
		}
	}
}

impl From<Notification> for Value {
	fn from(notif: Notification) -> Self {
		serde_json::to_value(&notif).unwrap()
	}
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
	pub duration: i64,
}
impl Music {
	pub fn create_music_response(entry: Music) -> MusicResponse {
		let mut hasher = DefaultHasher::new();
		entry.artist.hash(&mut hasher);
		entry.album.hash(&mut hasher);
		let hash = hasher.finish();
		let img_uuid = Uuid::from_u64_pair(hash, hash);
		MusicResponse {
			id: entry.music_id.clone(),
			artist: entry.artist,
			title: entry.title,
			album: entry.album,
			genre: entry.genre,
			times_played: entry.times_played,
			duration: entry.duration,
			image_url: img_uuid.to_string(),
		}
	}
}
#[derive(Debug, Serialize, Deserialize)]
pub struct MusicResponse {
	pub id: String,
	pub artist: String,
	pub title: String,
	pub album: String,
	pub genre: String,
	pub times_played: i32,
	pub duration: i64,
	pub image_url: String,
}
