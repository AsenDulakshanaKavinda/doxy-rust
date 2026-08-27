use std::{net::SocketAddr, sync::Arc};

use axum::{Router, routing::get};

use crate::state::AppState;


mod middleware;
mod routes;
mod state;
mod telemetry;
pub mod db;

#[tokio::main]
async fn main() {
    telemetry::init_tracing();

    let state = Arc::new(AppState::new());

    let port = state.config.app_port;

    let app = middleware::tracing::apply_tracing_middleware(
        Router::new()
            .route("/", get("home"))
            .route("/health", axum::routing::get(routes::health::health_check))
            .with_state(state),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|err| panic!("Failed to bind to {addr}: {err}"));

    tracing::info!("Server starting on http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(routes::health::shutdown_signal())
        .await
        .unwrap_or_else(|err| panic!("Server error: {err}"));
}
