use std::fs;
use std::path::Path;

mod config;
mod core;
mod lobic_db;
mod mail;
mod routes;
mod schema;
mod utils;

use config::{COVER_IMG_STORAGE, server_ip, MUSIC_STORAGE, PLAYLIST_COVER_IMG_STORAGE, PORT, USER_PFP_STORAGE};
use core::{app_state::AppState, migrations::run_migrations};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
	create_storage_directories().expect("Failed to create storage directories");

	dotenv().ok();
	tracing_subscriber::fmt().pretty().init();

	let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
	run_migrations(&db_url);

	let app_state = AppState::new();

	let app = core::routes::configure_routes(app_state)
		.layer(axum::middleware::from_fn(core::server::logger))
		.layer(core::server::configure_cors());

	core::server::start_server(app, &server_ip(), &PORT).await;
}

fn create_storage_directories() -> std::io::Result<()> {
	// Create the base storage directory if it doesn't exist
	if !Path::new("storage/").exists() {
		fs::create_dir("storage/")?;
	}

	// Create subdirectories
	let subdirectories = [
		COVER_IMG_STORAGE,
		MUSIC_STORAGE,
		USER_PFP_STORAGE,
		PLAYLIST_COVER_IMG_STORAGE,
	];

	for dir in subdirectories {
		if !Path::new(dir).exists() {
			fs::create_dir_all(dir)?;
		}
	}

	Ok(())
}
