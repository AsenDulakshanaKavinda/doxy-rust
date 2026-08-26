use axum::{body::Body, http::Request, Router};
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{info_span, Level, Span};


/// Creates a tracing span for an incoming HTTP request.
///
/// Extracts the `x-request-id` header (if present) and records key HTTP request metadata
/// (`method`, `uri`, and HTTP `version`) onto the span context.
fn make_span(request: &Request<Body>) -> Span {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    info_span!(
        "request",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri(),
        version = ?request.version(),
    )
}


/// Applies request ID generation, HTTP tracing, and response header propagation middleware
/// to an [`axum::Router`].
///
/// # Middleware Layer Order
/// 1. **[`SetRequestIdLayer`]**: Generates a UUID v4 for incoming requests if no `x-request-id` header is present.
/// 2. **[`TraceLayer`]**: Encloses request handling in a tracing span (via [`make_span`]) and logs completion metrics upon response.
/// 3. **[`PropagateRequestIdLayer`]**: Copies the `x-request-id` header onto the outgoing HTTP response.
///

pub fn apply_tracing_middleware<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.layer(
        ServiceBuilder::new()
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(make_span)
                    .on_response(DefaultOnResponse::new().level(Level::INFO).latency_unit(LatencyUnit::Millis)),
            )
            .layer(PropagateRequestIdLayer::x_request_id()),
    )
}