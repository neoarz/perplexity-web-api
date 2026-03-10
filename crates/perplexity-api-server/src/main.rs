use perplexity_api_server::{app, config, telemetry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    telemetry::init();

    let config = config::Config::from_env().map_err(|e| {
        tracing::error!("{e}");
        e
    })?;

    app::run(config).await
}
