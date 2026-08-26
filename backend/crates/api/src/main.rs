use std::{net::SocketAddr, sync::Arc};

use axum::Router;

use crate::state::AppState;

mod routes;
mod state;

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new());
    let port = state.config.app_port;

    let app = Router::new()
        .route("/health", axum::routing::get(routes::health::health_check))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|err| panic!("Failed to bind to {addr}: {err}"));

    println!("Server running on http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(routes::health::shutdown_signal())
        .await
        .unwrap_or_else(|err| panic!("Server error: {err}"));
}
