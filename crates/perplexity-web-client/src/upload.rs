use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const DEFAULT_UPLOAD_SOURCE: &str = "default";

#[derive(Debug, Clone)]
pub struct UploadAttachment {
    pub filename: String,
    pub content_type: String,
    pub bytes: Bytes,
}

impl UploadAttachment {
    pub fn new(
        filename: impl Into<String>,
        content_type: impl Into<String>,
        bytes: impl Into<Bytes>,
    ) -> Self {
        Self {
            filename: filename.into(),
            content_type: content_type.into(),
            bytes: bytes.into(),
        }
    }

    pub fn size_bytes(&self) -> usize {
        self.bytes.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadedAttachment {
    pub url: String,
    pub file_uuid: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct BatchCreateUploadUrlsRequest<'a> {
    pub files: BTreeMap<String, CreateUploadUrlRequest<'a>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CreateUploadUrlRequest<'a> {
    pub filename: &'a str,
    pub content_type: &'a str,
    pub source: &'static str,
    pub file_size: usize,
    pub force_image: bool,
    pub skip_parsing: bool,
    pub persistent_upload: bool,
}

impl<'a> CreateUploadUrlRequest<'a> {
    pub(crate) fn from_attachment(attachment: &'a UploadAttachment) -> Self {
        Self {
            filename: &attachment.filename,
            content_type: &attachment.content_type,
            source: DEFAULT_UPLOAD_SOURCE,
            file_size: attachment.size_bytes(),
            force_image: true,
            skip_parsing: false,
            persistent_upload: false,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct BatchCreateUploadUrlsResponse {
    pub results: BTreeMap<String, UploadUrlResponse>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UploadUrlResponse {
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub rate_limited: bool,
    #[serde(default)]
    pub file_uuid: Option<String>,
    #[serde(default)]
    pub s3_bucket_url: Option<String>,
    #[serde(default)]
    pub fields: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StorageUploadResponse {
    #[serde(default)]
    pub secure_url: Option<String>,
    #[serde(default)]
    pub eager: Vec<StorageUploadAsset>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StorageUploadAsset {
    #[serde(default)]
    pub secure_url: Option<String>,
}

pub(crate) fn fallback_asset_url(s3_bucket_url: &str, key: &str, filename: &str) -> String {
    format!("{s3_bucket_url}{key}").replace("${filename}", filename)
}
