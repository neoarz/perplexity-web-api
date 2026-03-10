use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidRequest,
    InvalidModel,
    Unauthorized,
    PerplexityError,
    UpstreamTimeout,
    InternalError,
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorDetail {
    pub code: ErrorCode,
    pub message: String,
}

impl ApiError {
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: ErrorCode::InvalidRequest,
            message: normalize_message(message.into()),
        }
    }

    pub fn invalid_model(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: ErrorCode::InvalidModel,
            message: normalize_message(message.into()),
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: ErrorCode::Unauthorized,
            message: normalize_message(message.into()),
        }
    }

    pub fn perplexity_error(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            code: ErrorCode::PerplexityError,
            message: normalize_message(message.into()),
        }
    }

    pub fn upstream_timeout(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::GATEWAY_TIMEOUT,
            code: ErrorCode::UpstreamTimeout,
            message: normalize_message(message.into()),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: ErrorCode::InternalError,
            message: normalize_message(message.into()),
        }
    }

    pub fn from_client_error(err: perplexity_web_client::Error) -> Self {
        match &err {
            perplexity_web_client::Error::Timeout(_) => Self::upstream_timeout(err.to_string()),
            perplexity_web_client::Error::Server { .. } => Self::perplexity_error(err.to_string()),
            _ => Self::perplexity_error(err.to_string()),
        }
    }

    pub fn body(&self) -> ErrorBody {
        ErrorBody {
            error: ErrorDetail {
                code: self.code.clone(),
                message: self.message.clone(),
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, axum::Json(self.body())).into_response()
    }
}

// A lion does concern himself with ugly error messages
fn normalize_message(message: String) -> String {
    let mut chars = message.chars();
    let Some(first) = chars.next() else {
        return message;
    };

    let mut normalized = String::new();
    normalized.extend(first.to_uppercase());
    normalized.push_str(chars.as_str());
    normalized
}
