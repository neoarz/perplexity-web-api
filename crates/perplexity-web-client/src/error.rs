use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("couldn't create the HTTP client: {0}")]
    HttpClientInit(#[source] rquest::Error),

    #[error("couldn't start the Perplexity session: {0}")]
    SessionWarmup(#[source] rquest::Error),

    #[error("request to Perplexity failed: {0}")]
    SearchRequest(#[source] rquest::Error),

    #[error("couldn't request attachment upload URLs: {0}")]
    AttachmentUploadRequest(#[source] rquest::Error),

    #[error("couldn't upload the attachment to storage: {0}")]
    AttachmentStorageUpload(#[source] rquest::Error),

    #[error("couldn't read the JSON response: {0}")]
    Json(#[from] serde_json::Error),

    #[error("request took longer than {0:?}")]
    Timeout(Duration),

    #[error("invalid UTF-8 in the stream")]
    InvalidUtf8,

    #[error("perplexity returned {status}: {message}")]
    Server { status: u16, message: String },

    #[error("perplexity rejected the upload: {0}")]
    UploadRejected(String),

    #[error("upload response was missing {0}")]
    InvalidUploadResponse(String),

    #[error("stream ended before anything came back")]
    UnexpectedEndOfStream,
}

pub type Result<T> = std::result::Result<T, Error>;
