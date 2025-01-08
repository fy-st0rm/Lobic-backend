/*
 * TODO:
 * [ ] Implement Leave and Delete lobby
 * [ ] Implement auto deletion when host disconnects
 * [ ] Implement verify route as middleware (if needed)
*/

mod api;
mod app_state;
mod config;
mod lobby;
mod lobic_db;
mod routes;
mod schema;
mod user_pool;
mod utils;

use api::migrations::run_migrations;
use app_state::AppState;
use config::{IP, PORT};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
	dotenv().ok();
	tracing_subscriber::fmt().pretty().init();

	let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
	run_migrations(&db_url);

	// Creating the global state
	let app_state = AppState::new();

	// Configure routes
	let app = api::routes::configure_routes(app_state)
		.layer(axum::middleware::from_fn(api::server::logger))
		.layer(api::server::configure_cors());

	// Start the server
	api::server::start_server(app, &IP, &PORT).await;
}
