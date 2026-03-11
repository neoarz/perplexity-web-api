use crate::error::ApiError;
use axum::body::Bytes;
use axum::http::HeaderMap;
use axum::http::header::CONTENT_TYPE;
use serde::de::DeserializeOwned;

pub fn parse_json_request<T>(headers: &HeaderMap, body: &Bytes) -> Result<T, ApiError>
where
    T: DeserializeOwned,
{
    validate_json_content_type(headers)?;
    serde_json::from_slice(body).map_err(|err| ApiError::invalid_request(err.to_string()))
}

fn validate_json_content_type(headers: &HeaderMap) -> Result<(), ApiError> {
    let Some(value) = headers.get(CONTENT_TYPE) else {
        return Err(ApiError::invalid_request(
            "content-type is missing, use application/json",
        ));
    };

    let value = value
        .to_str()
        .map_err(|_| ApiError::invalid_request("content-type header isn't valid"))?;
    let mime = value.split(';').next().map(str::trim).unwrap_or_default();

    if mime.eq_ignore_ascii_case("application/json") {
        Ok(())
    } else {
        Err(ApiError::invalid_request(
            "unsupported content-type, use application/json",
        ))
    }
}
