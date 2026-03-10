use perplexity_web_client::SearchWebResult;
use serde::Serialize;

/// Response body returned by `POST /v1/search`.
#[derive(Debug, Serialize)]
pub struct SearchApiResponse {
    /// Server-generated request id.
    pub id: String,
    /// Mode that was used for the request.
    pub mode: String,
    /// Model name that actually ran.
    pub model: String,
    /// Final answer text, if Perplexity returned one.
    pub answer: Option<String>,
    /// Final list of sources.
    pub web_results: Vec<SearchWebResult>,
    /// Values the caller can reuse for a follow-up request.
    pub follow_up: FollowUpResponse,
}

/// Follow-up data returned with a completed response.
#[derive(Debug, Serialize)]
pub struct FollowUpResponse {
    /// Conversation id for the next turn.
    pub backend_uuid: Option<String>,
    /// Attachments that should carry into the next turn.
    pub attachments: Vec<String>,
}

/// Response body returned by `GET /v1/models`.
#[derive(Debug, Serialize)]
pub struct ModelsApiResponse {
    /// Search models you can use with `mode = "search"`.
    pub search: Vec<ModelInfo>,
    /// Reasoning models you can use with `mode = "reason"`.
    pub reason: Vec<ModelInfo>,
    /// Fixed research mode information.
    pub research: ResearchModelInfo,
    /// Current defaults chosen by the server.
    pub defaults: ModelDefaults,
}

/// One model entry in the models list.
#[derive(Debug, Serialize)]
pub struct ModelInfo {
    /// Name accepted by the API.
    pub name: &'static str,
    /// Short human-readable label.
    pub description: &'static str,
}

/// Description for the fixed deep research mode.
#[derive(Debug, Serialize)]
pub struct ResearchModelInfo {
    /// Internal model name.
    pub name: &'static str,
    /// Short human-readable label.
    pub description: &'static str,
}

/// Default model names the server will use when the request leaves `model` unset.
#[derive(Debug, Serialize)]
pub struct ModelDefaults {
    /// Default search model.
    pub search: String,
    /// Default reasoning model.
    pub reason: String,
}

/// Payload sent in each SSE `message` event.
#[derive(Debug, Serialize)]
pub struct StreamEventPayload {
    /// Latest answer snapshot.
    pub answer: Option<String>,
    /// Latest source snapshot.
    pub web_results: Vec<SearchWebResult>,
}
