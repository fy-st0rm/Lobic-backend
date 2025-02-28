use axum::{
	body::Body,
	extract::Request,
	http::{header, Method},
	middleware::Next,
	response::Response,
	Router,
};
use colored::*;
use std::time::Instant;
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn configure_cors() -> CorsLayer {
	CorsLayer::new()
		.allow_origin(AllowOrigin::predicate(crate::config::allowed_origins))
		.allow_credentials(true)
		.allow_methods([Method::GET, Method::POST, Method::OPTIONS])
		.allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}

pub async fn start_server(app: Router, ip: &str, port: &str) {
	println!(
		"{}: {}",
		"Server hosted at".green(),
		format!("http://{ip}:{port}").cyan()
	);

	let listener = tokio::net::TcpListener::bind(format!("{ip}:{port}")).await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

pub async fn logger(req: Request<Body>, next: Next) -> Response {
	let start = Instant::now();
	let method = req.method().to_string();
	let uri = req.uri().to_string();

	let response = next.run(req).await;

	let colored_method = match method.as_str() {
		"GET" => method.bright_green(),
		"POST" => method.bright_yellow(),
		"PUT" => method.bright_blue(),
		"DELETE" => method.bright_red(),
		_ => method.normal(),
	};

	let status = response.status();
	let colored_status = if status.is_success() {
		status.as_u16().to_string().green()
	} else if status.is_client_error() {
		status.as_u16().to_string().yellow()
	} else if status.is_server_error() {
		status.as_u16().to_string().red()
	} else {
		status.as_u16().to_string().normal()
	};

	println!(
		"{:<6} {:<20} | status: {:<4} | latency: {:<10.2?}",
		colored_method,
		uri.bright_white(),
		colored_status,
		start.elapsed()
	);

	response
}
