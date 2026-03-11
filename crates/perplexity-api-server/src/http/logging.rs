use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use std::time::Instant;

pub async fn request_log(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request
        .uri()
        .path_and_query()
        .map(|value| value.as_str().to_owned())
        .unwrap_or_else(|| request.uri().path().to_owned());
    let started_at = Instant::now();

    let response = next.run(request).await;
    let status = response.status().as_u16();
    let should_skip = status == 200 && path.starts_with("/v1/search/stream");

    if !should_skip {
        log_request(method.as_ref(), &path, status, started_at);
    }

    response
}

pub fn log_request(method: &str, path: &str, status: u16, started_at: Instant) {
    let elapsed_ms = started_at.elapsed().as_secs_f64() * 1000.0;
    tracing::info!("{method} {path} {status} {elapsed_ms:.2}ms");
}
