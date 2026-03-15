use perplexity_web_client::{GeneratedImage, SearchWebResult, UploadedAttachment};
use serde::Serialize;

/// Response body returned by `POST /v1/search`
#[derive(Debug, Serialize)]
pub struct SearchApiResponse {
    /// Server-generated request id
    pub id: String,
    /// Mode that was used for the request
    pub mode: &'static str,
    /// Model name that actually ran
    pub model: String,
    /// Final answer text, if Perplexity returned one
    pub answer: Option<String>,
    /// Final list of sources
    pub web_results: Vec<SearchWebResult>,
    /// Values the caller can reuse for a follow-up request
    pub follow_up: FollowUpResponse,
}

/// Follow-up data returned with a completed response
#[derive(Debug, Serialize)]
pub struct FollowUpResponse {
    /// Conversation id for the next turn
    pub backend_uuid: Option<String>,
    /// Attachments that should carry into the next turn
    pub attachments: Vec<String>,
}

/// One uploaded attachment returned by `POST /v1/attachments`
#[derive(Debug, Serialize)]
pub struct UploadedAttachmentResponse {
    /// Direct attachment URL to send back into `/v1/search.attachments`
    pub url: String,
    /// Upstream Perplexity file uuid
    pub file_uuid: String,
    /// Original filename
    pub filename: String,
    /// MIME type used for the upload
    pub content_type: String,
    /// File size in bytes
    pub size_bytes: usize,
}

impl From<UploadedAttachment> for UploadedAttachmentResponse {
    fn from(attachment: UploadedAttachment) -> Self {
        Self {
            url: attachment.url,
            file_uuid: attachment.file_uuid,
            filename: attachment.filename,
            content_type: attachment.content_type,
            size_bytes: attachment.size_bytes,
        }
    }
}

/// Response body returned by `POST /v1/attachments`
#[derive(Debug, Serialize)]
pub struct AttachmentUploadApiResponse {
    /// Uploaded attachments in request order
    pub attachments: Vec<UploadedAttachmentResponse>,
}

/// One generated image returned by `POST /v1/images`
#[derive(Debug, Serialize)]
pub struct GeneratedImageResponse {
    /// Direct image URL
    pub url: String,
    /// Smaller preview URL, if Perplexity returned one
    pub thumbnail_url: Option<String>,
    /// Download URL, if Perplexity returned one
    pub download_url: Option<String>,
    /// MIME type, if Perplexity returned one
    pub mime_type: Option<String>,
    /// Upstream image source or router name
    pub source: Option<String>,
    /// Upstream generation model name
    pub generation_model: Option<String>,
    /// Prompt or prompt-like description of the generated image
    pub prompt: Option<String>,
}

impl From<GeneratedImage> for GeneratedImageResponse {
    fn from(image: GeneratedImage) -> Self {
        Self {
            url: image.url,
            thumbnail_url: image.thumbnail_url,
            download_url: image.download_url,
            mime_type: image.mime_type,
            source: image.source,
            generation_model: image.generation_model,
            prompt: image.prompt,
        }
    }
}

/// Response body returned by `POST /v1/images`
#[derive(Debug, Serialize)]
pub struct ImageApiResponse {
    /// Server-generated request id
    pub id: String,
    /// Model name that actually ran
    pub model: String,
    /// Original generation prompt
    pub prompt: String,
    /// Whether the upstream request produced generated images
    pub image_generation: bool,
    /// Generated image assets returned by Perplexity
    pub images: Vec<GeneratedImageResponse>,
    /// Final answer text, if Perplexity returned one
    pub answer: Option<String>,
    /// Values the caller can reuse for a follow-up request
    pub follow_up: FollowUpResponse,
}

/// Response body returned by `GET /v1/models`
#[derive(Debug, Serialize)]
pub struct ModelsApiResponse {
    /// Search models you can use with `mode = "search"`
    pub search: Vec<ModelInfo>,
    /// Reasoning models you can use with `mode = "reason"`
    pub reason: Vec<ModelInfo>,
    /// Fixed research mode information
    pub research: ResearchModelInfo,
    /// Current defaults chosen by the server
    pub defaults: ModelDefaults,
}

/// One model entry in the models list
#[derive(Debug, Serialize)]
pub struct ModelInfo {
    /// Name accepted by the API
    pub name: &'static str,
    /// Short human-readable label
    pub description: &'static str,
}

/// Description for the fixed deep research mode
#[derive(Debug, Serialize)]
pub struct ResearchModelInfo {
    /// Internal model name
    pub name: &'static str,
    /// Short human-readable label
    pub description: &'static str,
}

/// Default model names the server will use when the request leaves `model` unset
#[derive(Debug, Serialize)]
pub struct ModelDefaults {
    /// Default search model
    pub search: String,
    /// Default reasoning model
    pub reason: String,
}

/// Payload sent in each SSE `message` event
#[derive(Debug, Serialize)]
pub struct StreamEventPayload {
    /// Latest answer snapshot
    pub answer: Option<String>,
    /// Latest source snapshot
    pub web_results: Vec<SearchWebResult>,
}
