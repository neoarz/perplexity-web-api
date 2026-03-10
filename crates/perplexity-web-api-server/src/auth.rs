use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use std::sync::Arc;
use subtle::ConstantTimeEq;

pub async fn bearer_auth(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let Some(ref expected) = state.api_key else {
        return Ok(next.run(request).await);
    };

    let header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    let Some(provided) = header.and_then(|h| h.strip_prefix("Bearer ")) else {
        return Err(ApiError::unauthorized(
            "Missing or invalid Authorization header",
        ));
    };

    let is_valid: bool = provided.as_bytes().ct_eq(expected.as_bytes()).into();

    if is_valid {
        Ok(next.run(request).await)
    } else {
        Err(ApiError::unauthorized("That API key doesn't match"))
    }
}
