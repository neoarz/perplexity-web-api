//! Small Rust client for Perplexity's web app endpoints.
//!
//! It gives you a typed request/response layer on top of the browser-session
//! flow this project relies on.

mod auth;
mod client;
mod config;
mod error;
mod model;
mod parse;
mod request;
mod response;
mod session;
mod sse;
mod upload;

pub use auth::{AuthCookies, CSRF_TOKEN_COOKIE, SESSION_TOKEN_COOKIE};
pub use client::{Client, ClientBuilder};
pub use error::{Error, Result};
pub use model::{ModelPreference, ReasonModel, SearchModel};
pub use request::{FollowUpContext, SearchMode, SearchRequest, Source};
pub use response::{GeneratedImage, SearchEvent, SearchResponse, SearchWebResult};
pub use upload::{UploadAttachment, UploadedAttachment};
