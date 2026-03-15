use crate::api::model;
use crate::api::request::ApiMode;
use crate::api::request::ImageApiRequest;
use crate::api::response::{FollowUpResponse, GeneratedImageResponse, ImageApiResponse};
use crate::error::ApiError;
use crate::http::request::parse_json_request;
use crate::http::response::{JsonOutputQuery, json_response};
use crate::state::AppState;
use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::Response;
use std::sync::Arc;
use uuid::Uuid;

pub async fn images(
    State(state): State<Arc<AppState>>,
    Query(output): Query<JsonOutputQuery>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ApiError> {
    let pretty = output.pretty_enabled();
    let body: ImageApiRequest =
        parse_json_request(&headers, &body).map_err(|err| err.with_pretty(pretty))?;
    let resolved =
        model::resolve_image_request(body, &state).map_err(|err| err.with_pretty(pretty))?;
    let timeout = state.timeout_for_mode(ApiMode::Search);

    let result = tokio::time::timeout(timeout, state.service.search(resolved.search_request))
        .await
        .map_err(|_| {
            ApiError::upstream_timeout(format!("the request took longer than {timeout:?}"))
                .with_pretty(pretty)
        })?
        .map_err(|err| ApiError::from_client_error(err).with_pretty(pretty))?;

    if result.generated_images.is_empty() {
        return Err(ApiError::perplexity_error(
            "the upstream response did not include generated images",
        )
        .with_pretty(pretty));
    }

    let id = format!("req_{}", Uuid::new_v4());
    let images = result
        .generated_images
        .into_iter()
        .map(GeneratedImageResponse::from)
        .collect();

    Ok(json_response(
        StatusCode::OK,
        &ImageApiResponse {
            id,
            model: resolved.model_str,
            prompt: resolved.prompt,
            image_generation: result.image_generation,
            images,
            answer: result.answer,
            follow_up: FollowUpResponse {
                backend_uuid: result.follow_up.backend_uuid,
                attachments: result.follow_up.attachments,
            },
        },
        pretty,
    ))
}
