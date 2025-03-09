// @generated automatically by Diesel CLI.

diesel::table! {
    liked_songs (user_id, music_id) {
        user_id -> Text,
        music_id -> Text,
        song_added_date_time -> Text,
    }
}

diesel::table! {
    music (music_id) {
        music_id -> Text,
        artist -> Text,
        title -> Text,
        album -> Text,
        genre -> Text,
        times_played -> Integer,
        duration -> BigInt,
    }
}

diesel::table! {
    notifications (id) {
        id -> Text,
        user_id -> Text,
        op_code -> Text,
        value -> Text,
    }
}

diesel::table! {
    play_log (user_id, music_id) {
        user_id -> Text,
        music_id -> Text,
        music_played_date_time -> Text,
        user_times_played -> Integer,
    }
}

diesel::table! {
    playlist_shares (playlist_id, contributor_user_id) {
        playlist_id -> Text,
        contributor_user_id -> Text,
    }
}

diesel::table! {
    playlist_songs (playlist_id, music_id) {
        playlist_id -> Text,
        music_id -> Text,
        song_adder_id -> Text,
        song_added_date_time -> Text,
    }
}

diesel::table! {
    playlists (playlist_id) {
        playlist_id -> Text,
        playlist_name -> Text,
        user_id -> Text,
        creation_date_time -> Text,
        last_updated_date_time -> Text,
        is_playlist_combined -> Bool,
    }
}

diesel::table! {
    user_friendship (user_id, friend_id) {
        user_id -> Text,
        friend_id -> Text,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Text,
        username -> Text,
        email -> Text,
        pwd_hash -> Text,
        email_verified -> Bool,
        otp -> Text,
        otp_expires_at -> Text,
        otp_verified -> Nullable<Text>,
    }
}

diesel::joinable!(liked_songs -> music (music_id));
diesel::joinable!(liked_songs -> users (user_id));
diesel::joinable!(notifications -> users (user_id));
diesel::joinable!(play_log -> music (music_id));
diesel::joinable!(play_log -> users (user_id));
diesel::joinable!(playlist_shares -> playlists (playlist_id));
diesel::joinable!(playlist_shares -> users (contributor_user_id));
diesel::joinable!(playlist_songs -> music (music_id));
diesel::joinable!(playlist_songs -> playlists (playlist_id));
diesel::joinable!(playlist_songs -> users (song_adder_id));
diesel::joinable!(playlists -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    liked_songs,
    music,
    notifications,
    play_log,
    playlist_shares,
    playlist_songs,
    playlists,
    user_friendship,
    users,
);
