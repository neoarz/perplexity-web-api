use serde::Deserialize;

/// JSON body for `POST /v1/search` and `POST /v1/search/stream`.
#[derive(Debug, Deserialize)]
pub struct SearchApiRequest {
    /// The prompt or question to send to Perplexity.
    pub query: String,
    /// Request mode: `search`, `reason`, or `research`.
    #[serde(default = "default_mode")]
    pub mode: String,
    /// Optional model override for the chosen mode.
    #[serde(default)]
    pub model: Option<String>,
    /// Optional source filters. Empty falls back to `["web"]`.
    #[serde(default = "default_sources")]
    pub sources: Vec<String>,
    /// Language code sent upstream.
    #[serde(default = "default_language")]
    pub language: String,
    /// Whether to ask Perplexity in incognito mode.
    #[serde(default = "default_incognito")]
    pub incognito: bool,
    /// Follow-up data from an earlier response.
    #[serde(default)]
    pub follow_up: Option<FollowUpRequest>,
}

/// Follow-up values a client can send back on the next request.
#[derive(Debug, Deserialize)]
pub struct FollowUpRequest {
    /// Conversation id from the previous response.
    pub backend_uuid: Option<String>,
    /// Attachment URLs that should stay attached to the conversation.
    #[serde(default)]
    pub attachments: Vec<String>,
}

fn default_mode() -> String {
    "search".to_string()
}

fn default_sources() -> Vec<String> {
    vec!["web".to_string()]
}

fn default_language() -> String {
    "en-US".to_string()
}

fn default_incognito() -> bool {
    true
}
