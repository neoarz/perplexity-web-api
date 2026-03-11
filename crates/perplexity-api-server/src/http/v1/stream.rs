use crate::api::model;
use crate::api::request::SearchApiRequest;
use crate::api::response::{FollowUpResponse, SearchApiResponse, StreamEventPayload};
use crate::error::ApiError;
use crate::http::logging;
use crate::http::request::parse_json_request;
use crate::state::AppState;
use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::StreamExt;
use futures_util::stream::BoxStream;
use perplexity_web_client::SearchEvent;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Default, Deserialize)]
pub struct StreamOutputQuery {
    pub human: Option<String>,
}

impl StreamOutputQuery {
    fn human_enabled(&self) -> bool {
        self.human.as_deref() == Some("1")
    }
}

struct StreamContext {
    timeout: Duration,
    id: String,
    mode: &'static str,
    model: String,
    started_at: Instant,
    human_output: bool,
}

enum MessageSendResult {
    Emitted,
    Suppressed,
    ClientDisconnected,
}

pub async fn search_stream(
    State(state): State<Arc<AppState>>,
    Query(output): Query<StreamOutputQuery>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let started_at = Instant::now();
    let pretty = crate::http::response::DEFAULT_PRETTY_JSON;
    let body: SearchApiRequest =
        parse_json_request(&headers, &body).map_err(|err| err.with_pretty(pretty))?;
    let resolved = model::resolve(body, &state).map_err(|err| err.with_pretty(pretty))?;
    let timeout = state.timeout_for_mode(resolved.api_mode);
    let context = StreamContext {
        timeout,
        id: format!("req_{}", Uuid::new_v4()),
        mode: resolved.mode_str,
        model: resolved.model_str.clone(),
        started_at,
        human_output: output.human_enabled(),
    };

    let client_stream = tokio::time::timeout(
        timeout,
        state.service.search_stream(resolved.search_request),
    )
    .await
    .map_err(|_| {
        ApiError::upstream_timeout(format!(
            "Setting up the stream took longer than {timeout:?}"
        ))
        .with_pretty(pretty)
    })?
    .map_err(|err| ApiError::from_client_error(err).with_pretty(pretty))?;

    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(64);

    tokio::spawn(async move {
        forward_stream(tx, client_stream, context).await;
    });

    let stream = futures_util::stream::unfold(rx, |mut rx| async {
        rx.recv().await.map(|event| (event, rx))
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

async fn forward_stream(
    tx: tokio::sync::mpsc::Sender<Result<Event, Infallible>>,
    client_stream: BoxStream<'static, Result<SearchEvent, perplexity_web_client::Error>>,
    context: StreamContext,
) {
    let mut stream = std::pin::pin!(client_stream);
    let mut last_event = None;
    let mut last_message_payload: Option<String> = None;

    loop {
        let next_event = tokio::time::timeout(context.timeout, stream.next()).await;
        let result = match next_event {
            Ok(result) => result,
            Err(_) => {
                let _ = send_error(
                    &tx,
                    ApiError::upstream_timeout(format!(
                        "The stream took longer than {:?}",
                        context.timeout
                    )),
                    context.human_output,
                )
                .await;
                logging::log_stream_result(
                    "/v1/search/stream",
                    context.started_at,
                    "stream_timeout",
                    false,
                );
                return;
            }
        };

        match result {
            Some(Ok(event)) => {
                let payload = StreamEventPayload {
                    answer: event.answer.clone(),
                    web_results: event.web_results.clone(),
                };

                match send_message_event(
                    &tx,
                    &payload,
                    context.human_output,
                    &mut last_message_payload,
                )
                .await
                {
                    Ok(MessageSendResult::Emitted) => {
                        last_event = Some(event);
                    }
                    Ok(MessageSendResult::Suppressed) => {}
                    Ok(MessageSendResult::ClientDisconnected) => {
                        logging::log_stream_result(
                            "/v1/search/stream",
                            context.started_at,
                            "client_disconnected",
                            false,
                        );
                        return;
                    }
                    Err(err) => {
                        let _ = send_error(&tx, err, context.human_output).await;
                        logging::log_stream_result(
                            "/v1/search/stream",
                            context.started_at,
                            "stream_error",
                            false,
                        );
                        return;
                    }
                }
            }
            Some(Err(err)) => {
                let _ =
                    send_error(&tx, ApiError::from_client_error(err), context.human_output).await;
                logging::log_stream_result(
                    "/v1/search/stream",
                    context.started_at,
                    "stream_error",
                    false,
                );
                return;
            }
            None => break,
        }
    }

    let Some(event) = last_event else {
        let _ = send_error(
            &tx,
            ApiError::perplexity_error("The stream closed before any events came through"),
            context.human_output,
        )
        .await;
        logging::log_stream_result(
            "/v1/search/stream",
            context.started_at,
            "empty_stream",
            false,
        );
        return;
    };

    let response = SearchApiResponse {
        id: context.id,
        mode: context.mode,
        model: context.model,
        answer: event.answer,
        web_results: event.web_results,
        follow_up: FollowUpResponse {
            backend_uuid: event.backend_uuid,
            attachments: event.attachments,
        },
    };

    if let Err(err) = send_event(&tx, "done", &response, context.human_output).await {
        let _ = send_error(&tx, err, context.human_output).await;
        logging::log_stream_result(
            "/v1/search/stream",
            context.started_at,
            "stream_error",
            false,
        );
        return;
    }

    logging::log_stream_result("/v1/search/stream", context.started_at, "stream_done", true);
}

async fn send_message_event(
    tx: &tokio::sync::mpsc::Sender<Result<Event, Infallible>>,
    payload: &StreamEventPayload,
    human_output: bool,
    last_payload: &mut Option<String>,
) -> Result<MessageSendResult, ApiError> {
    if human_output && payload.answer.is_none() && payload.web_results.is_empty() {
        return Ok(MessageSendResult::Suppressed);
    }

    let data = serialize_event_payload(payload, human_output, true).map_err(|err| {
        ApiError::internal(format!("Couldn't serialize the message event: {err}"))
    })?;

    if human_output && last_payload.as_deref() == Some(data.as_str()) {
        return Ok(MessageSendResult::Suppressed);
    }

    *last_payload = Some(data.clone());

    if tx
        .send(Ok(Event::default().event("message").data(data)))
        .await
        .is_ok()
    {
        Ok(MessageSendResult::Emitted)
    } else {
        Ok(MessageSendResult::ClientDisconnected)
    }
}

async fn send_event<T: Serialize>(
    tx: &tokio::sync::mpsc::Sender<Result<Event, Infallible>>,
    name: &str,
    value: &T,
    human_output: bool,
) -> Result<bool, ApiError> {
    let data = serialize_event_payload(value, human_output, false)
        .map_err(|err| ApiError::internal(format!("Couldn't serialize the {name} event: {err}")))?;

    Ok(tx
        .send(Ok(Event::default().event(name).data(data)))
        .await
        .is_ok())
}

async fn send_error(
    tx: &tokio::sync::mpsc::Sender<Result<Event, Infallible>>,
    error: ApiError,
    human_output: bool,
) -> bool {
    let data = match if human_output {
        serde_json::to_string_pretty(&error.body())
    } else {
        serde_json::to_string(&error.body())
    } {
        Ok(data) => data,
        Err(err) => {
            let fallback =
                ApiError::internal(format!("Couldn't serialize the error payload: {err}"));
            match if human_output {
                serde_json::to_string_pretty(&fallback.body())
            } else {
                serde_json::to_string(&fallback.body())
            } {
                Ok(data) => data,
                Err(_) => return false,
            }
        }
    };

    tx.send(Ok(Event::default().event("error").data(data)))
        .await
        .is_ok()
}

fn serialize_event_payload<T>(
    value: &T,
    human_output: bool,
    add_trailing_newline: bool,
) -> Result<String, serde_json::Error>
where
    T: Serialize,
{
    let mut data = if human_output {
        serde_json::to_string_pretty(value)?
    } else {
        serde_json::to_string(value)?
    };

    if human_output && add_trailing_newline {
        data.push('\n');
    }

    Ok(data)
}
