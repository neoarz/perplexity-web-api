use tracing_subscriber::{EnvFilter, fmt};

pub fn init() {
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .init();
}
