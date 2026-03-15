use crate::auth::AuthCookies;
use crate::config::{
    API_BASE_URL, API_VERSION, ENDPOINT_BATCH_CREATE_UPLOAD_URLS, ENDPOINT_SSE_ASK, MODE_CONCISE,
    MODE_COPILOT,
};
use crate::error::{Error, Result};
use crate::request::{AskParams, AskPayload, SearchMode, SearchRequest};
use crate::response::{GeneratedImage, SearchEvent, SearchResponse};
use crate::session;
use crate::sse::SseStream;
use crate::upload::{
    BatchCreateUploadUrlsRequest, BatchCreateUploadUrlsResponse, CreateUploadUrlRequest,
    StorageUploadResponse, UploadAttachment, UploadedAttachment, fallback_asset_url,
};
use futures_util::{Stream, StreamExt};
use rquest::Client as HttpClient;
use rquest::multipart::{Form, Part};
use std::collections::BTreeMap;
use std::time::Duration;
use uuid::Uuid;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Configures a [`Client`] before the HTTP session is created
#[must_use = "builders do nothing unless consumed"]
pub struct ClientBuilder {
    cookies: Option<AuthCookies>,
    timeout: Duration,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            cookies: None,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn cookies(mut self, cookies: AuthCookies) -> Self {
        self.cookies = Some(cookies);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub async fn build(self) -> Result<Client> {
        let http = session::build_http_client(self.cookies.as_ref())?;
        session::warmup(&http, self.timeout).await?;

        Ok(Client {
            http,
            timeout: self.timeout,
        })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Async client around Perplexity's web search endpoints
#[derive(Clone)]
pub struct Client {
    http: HttpClient,
    timeout: Duration,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        let mut stream = Box::pin(self.search_stream(request).await?);
        let mut accumulator = SearchAccumulator::default();

        while let Some(result) = stream.next().await {
            accumulator.push(result?);
        }

        accumulator.finish()
    }

    pub async fn search_stream(
        &self,
        request: SearchRequest,
    ) -> Result<impl Stream<Item = Result<SearchEvent>> + use<>> {
        let mut attachments = Vec::new();

        if let Some(ref follow_up) = request.follow_up {
            attachments.extend(follow_up.attachments.clone());
        }

        let mode_str = match request.mode {
            SearchMode::Auto => MODE_CONCISE,
            _ => MODE_COPILOT,
        };

        let model_pref = request
            .model_preference
            .map(|p| p.as_str())
            .unwrap_or_else(|| request.mode.default_preference());

        let sources_str: Vec<&'static str> = request.sources.iter().map(|s| s.as_str()).collect();

        let payload = AskPayload {
            query_str: &request.query,
            params: AskParams {
                attachments,
                frontend_context_uuid: Uuid::new_v4().to_string(),
                frontend_uuid: Uuid::new_v4().to_string(),
                is_incognito: request.incognito,
                language: &request.language,
                last_backend_uuid: request.follow_up.and_then(|f| f.backend_uuid),
                mode: mode_str,
                model_preference: model_pref,
                source: "default",
                sources: sources_str,
                version: API_VERSION,
            },
        };

        let request_fut = self
            .http
            .post(format!("{API_BASE_URL}{ENDPOINT_SSE_ASK}"))
            .json(&payload)
            .send();

        let response = tokio::time::timeout(self.timeout, request_fut)
            .await
            .map_err(|_| Error::Timeout(self.timeout))?
            .map_err(Error::SearchRequest)?
            .error_for_status()
            .map_err(|e| Error::Server {
                status: e.status().map(|s| s.as_u16()).unwrap_or(0),
                message: e.to_string(),
            })?;

        Ok(SseStream::new(response.bytes_stream()))
    }

    pub async fn upload_attachments(
        &self,
        attachments: Vec<UploadAttachment>,
    ) -> Result<Vec<UploadedAttachment>> {
        if attachments.is_empty() {
            return Ok(Vec::new());
        }

        let upload_urls = self.request_upload_urls(&attachments).await?;
        let mut uploaded = Vec::with_capacity(attachments.len());

        for (index, attachment) in attachments.into_iter().enumerate() {
            let key = upload_key(index);
            let upload = upload_urls
                .results
                .get(&key)
                .ok_or_else(|| Error::InvalidUploadResponse(format!("result for '{key}'")))?;

            if let Some(error) = &upload.error {
                return Err(Error::UploadRejected(error.clone()));
            }

            if upload.rate_limited {
                return Err(Error::UploadRejected("rate_limited".to_string()));
            }

            let file_uuid = upload
                .file_uuid
                .as_deref()
                .ok_or_else(|| Error::InvalidUploadResponse("file_uuid".to_string()))?;
            let s3_bucket_url = upload
                .s3_bucket_url
                .as_deref()
                .ok_or_else(|| Error::InvalidUploadResponse("s3_bucket_url".to_string()))?;
            let fields = upload
                .fields
                .as_ref()
                .ok_or_else(|| Error::InvalidUploadResponse("fields".to_string()))?;

            let url = self
                .upload_attachment_to_storage(&attachment, s3_bucket_url, fields)
                .await?;

            let size_bytes = attachment.size_bytes();
            uploaded.push(UploadedAttachment {
                url,
                file_uuid: file_uuid.to_string(),
                filename: attachment.filename,
                content_type: attachment.content_type,
                size_bytes,
            });
        }

        Ok(uploaded)
    }

    async fn request_upload_urls(
        &self,
        attachments: &[UploadAttachment],
    ) -> Result<BatchCreateUploadUrlsResponse> {
        let files = attachments
            .iter()
            .enumerate()
            .map(|(index, attachment)| {
                (
                    upload_key(index),
                    CreateUploadUrlRequest::from_attachment(attachment),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let payload = BatchCreateUploadUrlsRequest { files };

        let request_fut = self
            .http
            .post(format!("{API_BASE_URL}{ENDPOINT_BATCH_CREATE_UPLOAD_URLS}"))
            .json(&payload)
            .send();

        let response = tokio::time::timeout(self.timeout, request_fut)
            .await
            .map_err(|_| Error::Timeout(self.timeout))?
            .map_err(Error::AttachmentUploadRequest)?
            .error_for_status()
            .map_err(|e| Error::Server {
                status: e.status().map(|s| s.as_u16()).unwrap_or(0),
                message: e.to_string(),
            })?;

        let body = response
            .text()
            .await
            .map_err(Error::AttachmentUploadRequest)?;
        serde_json::from_str(&body).map_err(Error::Json)
    }

    async fn upload_attachment_to_storage(
        &self,
        attachment: &UploadAttachment,
        s3_bucket_url: &str,
        fields: &BTreeMap<String, String>,
    ) -> Result<String> {
        let key = fields
            .get("key")
            .ok_or_else(|| Error::InvalidUploadResponse("fields.key".to_string()))?;

        let mut form = Form::new();
        for (field, value) in fields {
            form = form.text(field.clone(), value.clone());
        }

        let part = Part::bytes(attachment.bytes.to_vec())
            .file_name(attachment.filename.clone())
            .mime_str(&attachment.content_type)
            .map_err(|err| Error::InvalidUploadResponse(err.to_string()))?;
        form = form.part("file", part);

        let request_fut = self.http.post(s3_bucket_url).multipart(form).send();

        let response = tokio::time::timeout(self.timeout, request_fut)
            .await
            .map_err(|_| Error::Timeout(self.timeout))?
            .map_err(Error::AttachmentStorageUpload)?
            .error_for_status()
            .map_err(|e| Error::Server {
                status: e.status().map(|s| s.as_u16()).unwrap_or(0),
                message: e.to_string(),
            })?;

        let body = response
            .text()
            .await
            .map_err(Error::AttachmentStorageUpload)?;
        if body.trim().is_empty() {
            return Ok(fallback_asset_url(s3_bucket_url, key, &attachment.filename));
        }

        let response: StorageUploadResponse = serde_json::from_str(&body).map_err(Error::Json)?;
        response
            .eager
            .iter()
            .find_map(|asset| asset.secure_url.clone())
            .or(response.secure_url)
            .ok_or_else(|| Error::InvalidUploadResponse("secure_url".to_string()))
    }
}

#[derive(Default)]
struct SearchAccumulator {
    last_event: Option<SearchEvent>,
    image_generation: bool,
    generated_images: Vec<GeneratedImage>,
}

impl SearchAccumulator {
    fn push(&mut self, mut event: SearchEvent) {
        self.image_generation |= event.image_generation;
        merge_generated_images(
            &mut self.generated_images,
            std::mem::take(&mut event.generated_images),
        );
        self.last_event = Some(event);
    }

    fn finish(self) -> Result<SearchResponse> {
        let mut event = self.last_event.ok_or(Error::UnexpectedEndOfStream)?;
        merge_generated_images(&mut event.generated_images, self.generated_images);
        event.image_generation |= self.image_generation || !event.generated_images.is_empty();

        let raw = serde_json::to_value(&event).map_err(Error::Json)?;
        let follow_up = event.as_follow_up();

        Ok(SearchResponse {
            answer: event.answer,
            web_results: event.web_results,
            image_generation: event.image_generation,
            generated_images: event.generated_images,
            follow_up,
            raw,
        })
    }
}

fn merge_generated_images(existing: &mut Vec<GeneratedImage>, incoming: Vec<GeneratedImage>) {
    for image in incoming {
        if let Some(current) = existing.iter_mut().find(|current| current.url == image.url) {
            merge_generated_image(current, image);
        } else {
            existing.push(image);
        }
    }
}

fn merge_generated_image(existing: &mut GeneratedImage, incoming: GeneratedImage) {
    merge_optional_field(&mut existing.thumbnail_url, incoming.thumbnail_url);
    merge_optional_field(&mut existing.download_url, incoming.download_url);
    merge_optional_field(&mut existing.mime_type, incoming.mime_type);
    merge_optional_field(&mut existing.source, incoming.source);
    merge_optional_field(&mut existing.generation_model, incoming.generation_model);
    merge_optional_field(&mut existing.prompt, incoming.prompt);
}

fn merge_optional_field(slot: &mut Option<String>, incoming: Option<String>) {
    if slot.is_none() {
        *slot = incoming;
    }
}

fn upload_key(index: usize) -> String {
    format!("file_{index}")
}
