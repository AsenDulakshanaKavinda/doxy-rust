use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initializes global application logging using `tracing`
///
/// configures a tracing registry with tow layers
/// 1. An environment filter reading from `RUST_LOG` (defaults to `debug` for `axum_server` and `tower_http`).
/// 2. A JSON-formatted stdout log layer for structured output.
/// 
/// Read log filter directives from `RUST_LOG`; fall back to default module levels if unset.
/// Format log events as structured JSON objects for log aggregators.
/// Set this configuration as the global default tracing dispatcher.
/// 

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}
