use crate::config::Config;
use crate::http::router::create_router;
use crate::service::PerplexityService;
use crate::shutdown::shutdown_signal;
use crate::state::AppState;
use perplexity_web_client::{AuthCookies, Client};
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let cookies = AuthCookies::new(&config.session_token, &config.csrf_token);
    let client_timeout = [
        config.search_timeout,
        config.reason_timeout,
        config.research_timeout,
    ]
    .into_iter()
    .max()
    .unwrap_or(config.research_timeout);

    tracing::info!("Starting Perplexity client");
    let client = Client::builder()
        .cookies(cookies)
        .timeout(client_timeout)
        .build()
        .await
        .map_err(|e| {
            tracing::error!("Couldn't start the Perplexity client: {e}");
            e
        })?;
    tracing::info!("Perplexity client is ready");

    let addr = config.bind_addr();
    let service: Arc<dyn PerplexityService> = Arc::new(client);

    let state = Arc::new(AppState {
        service,
        api_key: config.api_key,
        default_search_model: config.search_model,
        default_reason_model: config.reason_model,
        search_timeout: config.search_timeout,
        reason_timeout: config.reason_timeout,
        research_timeout: config.research_timeout,
    });

    let router = create_router(state);
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("Listening on {addr}");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shut down");
    Ok(())
}
