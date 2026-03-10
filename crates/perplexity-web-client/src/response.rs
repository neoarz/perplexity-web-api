use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::request::FollowUpContext;

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
    /// Values you can feed into the next request to continue the thread
    pub follow_up: FollowUpContext,
    /// Raw data from the last event after normalization
    pub raw: serde_json::Value,
}
