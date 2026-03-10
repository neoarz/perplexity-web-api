#[cfg(unix)]
pub async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    match signal(SignalKind::terminate()) {
        Ok(mut sigterm) => {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = sigterm.recv() => {}
            }
        }
        Err(err) => {
            tracing::warn!("Couldn't register the SIGTERM handler: {err}");
            if let Err(e) = tokio::signal::ctrl_c().await {
                tracing::warn!("Couldn't listen for SIGINT: {e}");
            }
        }
    }
}

#[cfg(not(unix))]
pub async fn shutdown_signal() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::warn!("Couldn't listen for the shutdown signal: {err}");
    }
}
