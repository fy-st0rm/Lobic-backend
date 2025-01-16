-- User
CREATE TABLE users (
	user_id TEXT PRIMARY KEY NOT NULL,
	username TEXT NOT NULL,
	email TEXT NOT NULL,
	pwd_hash TEXT NOT NULL
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
	times_played INTEGER NOT NULL
);

-- Playlists
CREATE TABLE playlists (
	playlist_id TEXT PRIMARY KEY NOT NULL,
	playlist_name TEXT NOT NULL,
	user_id TEXT NOT NULL REFERENCES users(user_id),
	description TEXT,
	creation_date_time TEXT NOT NULL,
	last_updated_date_time TEXT NOT NULL
);

CREATE TABLE playlist_songs (
	playlist_id TEXT NOT NULL REFERENCES playlists(playlist_id),
	music_id TEXT NOT NULL REFERENCES music(music_id),
	song_added_date_time TEXT NOT NULL,
	PRIMARY KEY (playlist_id, music_id)
);

CREATE TABLE playlist_shares (
	playlist_id TEXT NOT NULL REFERENCES playlists(playlist_id),
	owner_user_id TEXT NOT NULL REFERENCES users(user_id),
	shared_to_user_id TEXT NOT NULL REFERENCES users(user_id),
	share_permission TEXT NOT NULL,
	PRIMARY KEY (playlist_id, shared_to_user_id)
);

--for Recently Played of each user
CREATE TABLE play_log (
	user_id TEXT NOT NULL REFERENCES users(user_id),
	music_id TEXT NOT NULL REFERENCES music(music_id),
	music_played_date_time TEXT NOT NULL,
	PRIMARY KEY (user_id,music_id)
);


--TODO : favourites