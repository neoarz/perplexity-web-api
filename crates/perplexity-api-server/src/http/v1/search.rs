use crate::api::model;
use crate::api::request::SearchApiRequest;
use crate::api::response::{FollowUpResponse, SearchApiResponse};
use crate::error::ApiError;
use crate::http::request::parse_json_request;
use crate::state::AppState;
use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::HeaderMap;
use std::sync::Arc;
use uuid::Uuid;

pub async fn search(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<SearchApiResponse>, ApiError> {
    let body: SearchApiRequest = parse_json_request(&headers, &body)?;
    let resolved = model::resolve(body, &state)?;
    let timeout = state.timeout_for_mode(&resolved.mode_str);

    let result = tokio::time::timeout(timeout, state.service.search(resolved.search_request))
        .await
        .map_err(|_| {
            ApiError::upstream_timeout(format!("The request took longer than {timeout:?}"))
        })?
        .map_err(ApiError::from_client_error)?;

    let id = format!("req_{}", Uuid::new_v4());

    Ok(Json(SearchApiResponse {
        id,
        mode: resolved.mode_str,
        model: resolved.model_str,
        answer: result.answer,
        web_results: result.web_results,
        follow_up: FollowUpResponse {
            backend_uuid: result.follow_up.backend_uuid,
            attachments: result.follow_up.attachments,
        },
    }))
}
