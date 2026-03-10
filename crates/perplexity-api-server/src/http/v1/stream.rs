use crate::api::model;
use crate::api::request::SearchApiRequest;
use crate::api::response::{FollowUpResponse, SearchApiResponse, StreamEventPayload};
use crate::error::ApiError;
use crate::http::request::parse_json_request;
use crate::state::AppState;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::StreamExt;
use futures_util::stream::BoxStream;
use perplexity_web_client::SearchEvent;
use serde::Serialize;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub async fn search_stream(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let body: SearchApiRequest = parse_json_request(&headers, &body)?;
    let resolved = model::resolve(body, &state)?;
    let mode_str = resolved.mode_str.clone();
    let model_str = resolved.model_str.clone();
    let timeout = state.timeout_for_mode(&resolved.mode_str);
    let id = format!("req_{}", Uuid::new_v4());

    let client_stream = tokio::time::timeout(
        timeout,
        state.service.search_stream(resolved.search_request),
    )
    .await
    .map_err(|_| {
        ApiError::upstream_timeout(format!(
            "Setting up the stream took longer than {timeout:?}"
        ))
    })?
    .map_err(ApiError::from_client_error)?;

    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(64);

    tokio::spawn(async move {
        forward_stream(tx, client_stream, timeout, id, mode_str, model_str).await;
    });

    let stream = futures_util::stream::unfold(rx, |mut rx| async {
        rx.recv().await.map(|event| (event, rx))
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

async fn forward_stream(
    tx: tokio::sync::mpsc::Sender<Result<Event, Infallible>>,
    client_stream: BoxStream<'static, Result<SearchEvent, perplexity_web_client::Error>>,
    timeout: Duration,
    id: String,
    mode: String,
    model: String,
) {
    let mut stream = std::pin::pin!(client_stream);
    let mut last_event = None;

    loop {
        let next_event = tokio::time::timeout(timeout, stream.next()).await;
        let result = match next_event {
            Ok(result) => result,
            Err(_) => {
                let _ = send_error(
                    &tx,
                    ApiError::upstream_timeout(format!("The stream took longer than {timeout:?}")),
                )
                .await;
                return;
            }
        };

        match result {
            Some(Ok(event)) => {
                let payload = StreamEventPayload {
                    answer: event.answer.clone(),
                    web_results: event.web_results.clone(),
                };

                match send_event(&tx, "message", &payload).await {
                    Ok(true) => last_event = Some(event),
                    Ok(false) => return,
                    Err(err) => {
                        let _ = send_error(&tx, err).await;
                        return;
                    }
                }
            }
            Some(Err(err)) => {
                let _ = send_error(&tx, ApiError::from_client_error(err)).await;
                return;
            }
            None => break,
        }
    }

    let Some(event) = last_event else {
        let _ = send_error(
            &tx,
            ApiError::perplexity_error("The stream closed before any events came through"),
        )
        .await;
        return;
    };

    let response = SearchApiResponse {
        id,
        mode,
        model,
        answer: event.answer,
        web_results: event.web_results,
        follow_up: FollowUpResponse {
            backend_uuid: event.backend_uuid,
            attachments: event.attachments,
        },
    };

    if let Err(err) = send_event(&tx, "done", &response).await {
        let _ = send_error(&tx, err).await;
    }
}

async fn send_event<T: Serialize>(
    tx: &tokio::sync::mpsc::Sender<Result<Event, Infallible>>,
    name: &str,
    value: &T,
) -> Result<bool, ApiError> {
    let data = serde_json::to_string(value)
        .map_err(|err| ApiError::internal(format!("Couldn't serialize the {name} event: {err}")))?;

    Ok(tx
        .send(Ok(Event::default().event(name).data(data)))
        .await
        .is_ok())
}

async fn send_error(
    tx: &tokio::sync::mpsc::Sender<Result<Event, Infallible>>,
    error: ApiError,
) -> bool {
    let data = match serde_json::to_string(&error.body()) {
        Ok(data) => data,
        Err(err) => {
            let fallback =
                ApiError::internal(format!("Couldn't serialize the error payload: {err}"));
            match serde_json::to_string(&fallback.body()) {
                Ok(data) => data,
                Err(_) => return false,
            }
        }
    };

    tx.send(Ok(Event::default().event("error").data(data)))
        .await
        .is_ok()
}
