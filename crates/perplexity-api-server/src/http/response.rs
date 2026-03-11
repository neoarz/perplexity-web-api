use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde::Serialize;

pub const DEFAULT_PRETTY_JSON: bool = true;

#[derive(Debug, Default, Deserialize)]
pub struct JsonOutputQuery {
    pub pretty: Option<String>,
}

impl JsonOutputQuery {
    pub fn pretty_enabled(&self) -> bool {
        self.pretty
            .as_deref()
            .map_or(DEFAULT_PRETTY_JSON, |value| value != "0")
    }
}

pub fn json_response<T>(status: StatusCode, value: &T, pretty: bool) -> Response
where
    T: Serialize,
{
    let body = match serialize_json(value, pretty) {
        Ok(body) => body,
        Err(_) => {
            return internal_error_response("couldn't serialize the response", pretty);
        }
    };

    (
        status,
        [("content-type", "application/json; charset=utf-8")],
        body,
    )
        .into_response()
}

pub fn pretty_enabled_from_query(query: Option<&str>) -> bool {
    query
        .and_then(|query| {
            query
                .split('&')
                .find_map(|pair| pair.split_once('=').filter(|(key, _)| *key == "pretty"))
                .map(|(_, value)| value)
        })
        .map_or(DEFAULT_PRETTY_JSON, |value| value != "0")
}

pub fn serialize_json<T>(value: &T, pretty: bool) -> Result<Vec<u8>, serde_json::Error>
where
    T: Serialize,
{
    let mut body = if pretty {
        serde_json::to_vec_pretty(value)?
    } else {
        serde_json::to_vec(value)?
    };

    body.push(b'\n');
    Ok(body)
}

fn internal_error_response(message: &str, pretty: bool) -> Response {
    let fallback = serde_json::json!({
        "error": {
            "code": "internal_error",
            "message": message,
        }
    });

    let body = serialize_json(&fallback, pretty).unwrap_or_else(|_| {
        b"{\"error\":{\"code\":\"internal_error\",\"message\":\"couldn't serialize the error\"}}\n"
            .to_vec()
    });

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        [("content-type", "application/json; charset=utf-8")],
        body,
    )
        .into_response()
}
