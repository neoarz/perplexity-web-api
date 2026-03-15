use crate::auth::bearer_auth;
use crate::http::{health, logging, v1};
use crate::state::AppState;
use axum::Router;
use axum::middleware;
use axum::routing::{get, post};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub fn create_router(state: Arc<AppState>) -> Router {
    let authed_routes = Router::new()
        .route("/v1/models", get(v1::models::list_models))
        .route("/v1/images", post(v1::images::images))
        .route("/v1/search", post(v1::search::search))
        .route("/v1/search/stream", post(v1::stream::search_stream))
        .layer(middleware::from_fn_with_state(state.clone(), bearer_auth));

    Router::new()
        .route("/health", get(health::health))
        .merge(authed_routes)
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn(logging::request_log))
        .with_state(state)
}
