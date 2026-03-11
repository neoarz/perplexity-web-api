use crate::http::response::{DEFAULT_PRETTY_JSON, serialize_json};
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
    pub pretty: bool,
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
            message: message.into(),
            pretty: DEFAULT_PRETTY_JSON,
        }
    }

    pub fn invalid_model(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: ErrorCode::InvalidModel,
            message: message.into(),
            pretty: DEFAULT_PRETTY_JSON,
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: ErrorCode::Unauthorized,
            message: message.into(),
            pretty: DEFAULT_PRETTY_JSON,
        }
    }

    pub fn perplexity_error(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            code: ErrorCode::PerplexityError,
            message: message.into(),
            pretty: DEFAULT_PRETTY_JSON,
        }
    }

    pub fn upstream_timeout(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::GATEWAY_TIMEOUT,
            code: ErrorCode::UpstreamTimeout,
            message: message.into(),
            pretty: DEFAULT_PRETTY_JSON,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: ErrorCode::InternalError,
            message: message.into(),
            pretty: DEFAULT_PRETTY_JSON,
        }
    }

    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
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
        let body = serialize_json(&self.body(), self.pretty).unwrap_or_else(|_| {
            b"{\n  \"error\": {\n    \"code\": \"internal_error\",\n    \"message\": \"couldn't serialize the error\"\n  }\n}\n"
                .to_vec()
        });

        (
            self.status,
            [("content-type", "application/json; charset=utf-8")],
            body,
        )
            .into_response()
    }
}
