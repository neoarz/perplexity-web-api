use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiMode {
    #[default]
    Search,
    Reason,
    Research,
}

impl ApiMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Search => "search",
            Self::Reason => "reason",
            Self::Research => "research",
        }
    }
}

/// JSON body for `POST /v1/search` and `POST /v1/search/stream`
#[derive(Debug, Deserialize)]
pub struct SearchApiRequest {
    /// The prompt or question to send to Perplexity
    pub query: String,
    /// Request mode: `search`, `reason`, or `research`
    #[serde(default)]
    pub mode: ApiMode,
    /// Optional model override for the chosen mode
    #[serde(default)]
    pub model: Option<String>,
    /// Optional source filters. Empty falls back to `["web"]`
    #[serde(default = "default_sources")]
    pub sources: Vec<String>,
    /// Language code sent upstream
    #[serde(default = "default_language")]
    pub language: String,
    /// Whether to ask Perplexity in incognito mode
    #[serde(default = "default_incognito")]
    pub incognito: bool,
    /// Top-level attachment URLs for the request
    #[serde(default)]
    pub attachments: Vec<String>,
    /// Follow-up data from an earlier response
    #[serde(default)]
    pub follow_up: Option<FollowUpRequest>,
}

/// Follow-up values a client can send back on the next request
#[derive(Debug, Deserialize)]
pub struct FollowUpRequest {
    /// Conversation id from the previous response
    pub backend_uuid: Option<String>,
    /// Attachment URLs that should stay attached to the conversation
    #[serde(default)]
    pub attachments: Vec<String>,
}

/// JSON body for `POST /v1/images`
#[derive(Debug, Deserialize)]
pub struct ImageApiRequest {
    /// The image generation prompt to send to Perplexity
    pub prompt: String,
    /// Optional search-model override
    #[serde(default)]
    pub model: Option<String>,
    /// Language code sent upstream
    #[serde(default = "default_language")]
    pub language: String,
    /// Whether to ask Perplexity in incognito mode
    #[serde(default = "default_incognito")]
    pub incognito: bool,
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
