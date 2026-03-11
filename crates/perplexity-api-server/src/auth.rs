use crate::error::ApiError;
use crate::http::response::pretty_enabled_from_query;
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
    let pretty = pretty_enabled_from_query(request.uri().query());
    let Some(ref expected) = state.api_key else {
        return Ok(next.run(request).await);
    };

    let header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    let Some(provided) = header.and_then(parse_bearer_token) else {
        return Err(
            ApiError::unauthorized("missing or invalid authorization header").with_pretty(pretty),
        );
    };

    let is_valid: bool = provided.as_bytes().ct_eq(expected.as_bytes()).into();

    if is_valid {
        Ok(next.run(request).await)
    } else {
        Err(ApiError::unauthorized("api key does not match").with_pretty(pretty))
    }
}

fn parse_bearer_token(header: &str) -> Option<&str> {
    let (scheme, token) = header.split_once(' ')?;
    if scheme.eq_ignore_ascii_case("bearer") && !token.is_empty() {
        Some(token)
    } else {
        None
    }
}
