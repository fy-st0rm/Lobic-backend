-- User
CREATE TABLE users (
	user_id TEXT PRIMARY KEY NOT NULL,
	username TEXT NOT NULL,
	email TEXT NOT NULL,
	pwd_hash TEXT NOT NULL,
	email_verified BOOLEAN NOT NULL,
	otp TEXT NOT NULL,
	otp_expires_at TEXT NOT NULL,
	otp_verified TEXT
);

CREATE TABLE user_friendship (
	user_id TEXT NOT NULL REFERENCES users(user_id),
	friend_id TEXT NOT NULL REFERENCES users(user_id),
	PRIMARY KEY (user_id, friend_id)
);

-- Music
CREATE TABLE music (
	music_id TEXT PRIMARY KEY NOT NULL,
	artist TEXT NOT NULL,
	title TEXT NOT NULL,
	album TEXT NOT NULL,
	genre TEXT NOT NULL,
	times_played INTEGER NOT NULL,
	duration BIGINT NOT NULL
);

-- Playlists
CREATE TABLE playlists (
	playlist_id TEXT PRIMARY KEY NOT NULL,
	playlist_name TEXT NOT NULL,
	user_id TEXT NOT NULL REFERENCES users(user_id),
	creation_date_time TEXT NOT NULL,
	last_updated_date_time TEXT NOT NULL,
	is_playlist_combined BOOLEAN NOT NULL --0=solo 1=combined
);

CREATE TABLE playlist_songs (
	playlist_id TEXT NOT NULL REFERENCES playlists(playlist_id),
	music_id TEXT NOT NULL REFERENCES music(music_id),
	song_adder_id TEXT NOT NULL REFERENCES users(user_id), --for combined playlist added by
	song_added_date_time TEXT NOT NULL,
	PRIMARY KEY (playlist_id, music_id)
);

CREATE TABLE playlist_shares (
	playlist_id TEXT NOT NULL REFERENCES playlists(playlist_id),
	contributor_user_id TEXT NOT NULL REFERENCES users(user_id),
	--every contributor is the admin
	PRIMARY KEY (playlist_id,contributor_user_id)
);

--for Recently Played of each user
CREATE TABLE play_log (
	user_id TEXT NOT NULL REFERENCES users(user_id),
	music_id TEXT NOT NULL REFERENCES music(music_id),
	music_played_date_time TEXT NOT NULL,
	user_times_played INTEGER NOT NULL,
	PRIMARY KEY (user_id,music_id)
);

-- liked songs 
CREATE TABLE liked_songs(
	user_id TEXT NOT NULL REFERENCES users(user_id),
	music_id TEXT NOT NULL REFERENCES music(music_id),
	song_added_date_time TEXT NOT NULL,
	PRIMARY KEY (user_id, music_id)
);

-- notification
CREATE TABLE notifications(
	id TEXT PRIMARY KEY NOT NULL,
	user_id TEXT NOT NULL REFERENCES users(user_id),
	op_code TEXT NOT NULL,
	value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_play_log_user_music ON play_log(user_id, music_id);
CREATE INDEX IF NOT EXISTS idx_music_id ON music(music_id); 
