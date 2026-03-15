use crate::api::request::ApiMode;
use crate::api::response::{AttachmentUploadApiResponse, UploadedAttachmentResponse};
use crate::error::ApiError;
use crate::http::response::{JsonOutputQuery, json_response};
use crate::state::AppState;
use axum::extract::{Multipart, Query, State, multipart::MultipartRejection};
use axum::http::StatusCode;
use axum::response::Response;
use perplexity_web_client::UploadAttachment;
use std::sync::Arc;

const MAX_UPLOAD_FILES: usize = 10;
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

pub async fn upload_attachments(
    State(state): State<Arc<AppState>>,
    Query(output): Query<JsonOutputQuery>,
    multipart: Result<Multipart, MultipartRejection>,
) -> Result<Response, ApiError> {
    let pretty = output.pretty_enabled();
    let multipart = multipart.map_err(|err| {
        ApiError::invalid_request(format!("invalid multipart form-data: {err}")).with_pretty(pretty)
    })?;
    let attachments = collect_upload_attachments(multipart)
        .await
        .map_err(|err| err.with_pretty(pretty))?;

    if attachments.is_empty() {
        return Err(ApiError::invalid_request("at least one file is required").with_pretty(pretty));
    }

    let timeout = state.timeout_for_mode(ApiMode::Search);
    let uploaded = tokio::time::timeout(timeout, state.service.upload_attachments(attachments))
        .await
        .map_err(|_| {
            ApiError::upstream_timeout(format!("the request took longer than {timeout:?}"))
                .with_pretty(pretty)
        })?
        .map_err(|err| ApiError::from_client_error(err).with_pretty(pretty))?;

    Ok(json_response(
        StatusCode::OK,
        &AttachmentUploadApiResponse {
            attachments: uploaded
                .into_iter()
                .map(UploadedAttachmentResponse::from)
                .collect(),
        },
        pretty,
    ))
}

async fn collect_upload_attachments(
    mut multipart: Multipart,
) -> Result<Vec<UploadAttachment>, ApiError> {
    let mut attachments = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ApiError::invalid_request(format!("couldn't read multipart data: {err}")))?
    {
        if attachments.len() >= MAX_UPLOAD_FILES {
            return Err(ApiError::invalid_request(format!(
                "at most {MAX_UPLOAD_FILES} files are allowed"
            )));
        }

        let Some(name) = field.name() else {
            return Err(ApiError::invalid_request("multipart field name is missing"));
        };
        if name != "files" {
            return Err(ApiError::invalid_request(
                "unsupported multipart field, use repeated files fields",
            ));
        }

        let Some(filename) = field.file_name().map(ToString::to_string) else {
            return Err(ApiError::invalid_request(
                "uploaded file is missing a filename",
            ));
        };
        let declared_content_type = field.content_type().map(ToString::to_string);

        let bytes = read_bounded_file_bytes(field, &filename).await?;
        if bytes.is_empty() {
            return Err(ApiError::invalid_request(format!(
                "uploaded file '{filename}' is empty"
            )));
        }

        let content_type =
            resolve_image_content_type(&filename, declared_content_type.as_deref(), &bytes)?;
        attachments.push(UploadAttachment::new(filename, content_type, bytes));
    }

    Ok(attachments)
}

async fn read_bounded_file_bytes(
    mut field: axum::extract::multipart::Field<'_>,
    filename: &str,
) -> Result<Vec<u8>, ApiError> {
    let mut bytes = Vec::new();

    while let Some(chunk) = field.chunk().await.map_err(|err| {
        ApiError::invalid_request(format!("couldn't read uploaded file '{filename}': {err}"))
    })? {
        if bytes.len().saturating_add(chunk.len()) > MAX_FILE_SIZE {
            return Err(ApiError::invalid_request(format!(
                "uploaded file '{filename}' exceeds the {MAX_FILE_SIZE}-byte limit"
            )));
        }

        bytes.extend_from_slice(&chunk);
    }

    Ok(bytes)
}

fn resolve_image_content_type(
    filename: &str,
    declared_content_type: Option<&str>,
    bytes: &[u8],
) -> Result<String, ApiError> {
    if let Some(kind) = infer::get(bytes) {
        let mime = kind.mime_type();
        if mime.starts_with("image/") {
            return Ok(mime.to_string());
        }

        return Err(ApiError::invalid_request(format!(
            "uploaded file '{filename}' is not a supported image"
        )));
    }

    if let Some(content_type) =
        declared_content_type.filter(|content_type| content_type.starts_with("image/"))
    {
        return Ok(content_type.to_string());
    }

    let declared = declared_content_type.unwrap_or("unknown");
    Err(ApiError::invalid_request(format!(
        "unsupported content type '{declared}', use an image file"
    )))
}
