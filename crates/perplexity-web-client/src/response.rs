use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::request::FollowUpContext;

/// One generated image returned by Perplexity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedImage {
    /// Direct image URL
    pub url: String,
    /// Smaller preview URL, if Perplexity returned one
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    /// Download URL, if Perplexity returned one
    #[serde(default)]
    pub download_url: Option<String>,
    /// MIME type, if Perplexity returned one
    #[serde(default)]
    pub mime_type: Option<String>,
    /// Upstream image source or router name
    #[serde(default)]
    pub source: Option<String>,
    /// Upstream generation model name
    #[serde(default)]
    pub generation_model: Option<String>,
    /// Prompt or prompt-like description of the generated image
    #[serde(default)]
    pub prompt: Option<String>,
}

/// One parsed event from Perplexity's SSE stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEvent {
    /// Current answer text, if the event includes one
    #[serde(default)]
    pub answer: Option<String>,
    /// Sources that came with this event
    #[serde(default)]
    pub web_results: Vec<SearchWebResult>,
    /// Conversation id for follow-up requests
    #[serde(default)]
    pub backend_uuid: Option<String>,
    /// Attachments carried by this event
    #[serde(default)]
    pub attachments: Vec<String>,
    /// Whether this event indicates image generation
    #[serde(default)]
    pub image_generation: bool,
    /// Generated images carried by this event
    #[serde(default)]
    pub generated_images: Vec<GeneratedImage>,
    /// Everything else from the raw event payload
    #[serde(flatten)]
    pub raw: HashMap<String, serde_json::Value>,
}

impl SearchEvent {
    pub fn as_follow_up(&self) -> FollowUpContext {
        FollowUpContext {
            backend_uuid: self.backend_uuid.clone(),
            attachments: self.attachments.clone(),
        }
    }
}

/// One web result returned by Perplexity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchWebResult {
    /// Display name of the source
    pub name: String,
    /// Source URL
    pub url: String,
    /// Short snippet shown for the source
    pub snippet: String,
}

/// Final response returned by [`Client::search`](crate::Client::search).
#[derive(Debug, Clone)]
pub struct SearchResponse {
    /// Final answer text, if present
    pub answer: Option<String>,
    /// Final source list
    pub web_results: Vec<SearchWebResult>,
    /// Whether the upstream request generated images
    pub image_generation: bool,
    /// Final generated image list
    pub generated_images: Vec<GeneratedImage>,
    /// Values you can feed into the next request to continue the thread
    pub follow_up: FollowUpContext,
    /// Raw data from the last event after normalization
    pub raw: serde_json::Value,
}
