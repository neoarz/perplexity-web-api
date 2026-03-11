use crate::http::response::{JsonOutputQuery, json_response};
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::Response;
use serde_json::json;

pub async fn health(Query(output): Query<JsonOutputQuery>) -> Response {
    json_response(
        StatusCode::OK,
        &json!({"status": "ok"}),
        output.pretty_enabled(),
    )
}
