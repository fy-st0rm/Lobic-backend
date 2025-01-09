/*
 * TODO:
 * [ ] Implement Leave and Delete lobby
 * [ ] Implement auto deletion when host disconnects
 * [ ] Implement verify route as middleware (if needed)
*/

mod config;
mod core;
mod lobic_db;
mod routes;
mod schema;
mod utils;

use config::{IP, PORT};
use core::{app_state::AppState, migrations::run_migrations};
use dotenv::dotenv;
use std::fs;

#[tokio::main]
async fn main() {
	// Ensure the storage directory exists
	if !fs::metadata("storage/").is_ok() {
		fs::create_dir("storage/").expect("Failed to create storage directory");
	}

	dotenv().ok();
	tracing_subscriber::fmt().pretty().init();

	let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
	run_migrations(&db_url);

	// Creating the global state
	let app_state = AppState::new();

	// Configure routes
	let app = core::routes::configure_routes(app_state)
		.layer(axum::middleware::from_fn(core::server::logger))
		.layer(core::server::configure_cors());

	// Start the server
	core::server::start_server(app, &IP, &PORT).await;
}
