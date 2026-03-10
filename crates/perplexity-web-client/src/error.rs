use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't create the HTTP client: {0}")]
    HttpClientInit(#[source] rquest::Error),

    #[error("Couldn't start the Perplexity session: {0}")]
    SessionWarmup(#[source] rquest::Error),

    #[error("Request to Perplexity failed: {0}")]
    SearchRequest(#[source] rquest::Error),

    #[error("Couldn't read the JSON response: {0}")]
    Json(#[from] serde_json::Error),

    #[error("The request took longer than {0:?}")]
    Timeout(Duration),

    #[error("Got invalid UTF-8 in the stream")]
    InvalidUtf8,

    #[error("Perplexity returned {status}: {message}")]
    Server { status: u16, message: String },

    #[error("The stream ended before anything came back")]
    UnexpectedEndOfStream,
}

pub type Result<T> = std::result::Result<T, Error>;
