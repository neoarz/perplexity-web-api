use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing_subscriber::Layer;
use tracing_subscriber::fmt::writer::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, fmt, util::SubscriberInitExt};

pub fn init() {
    let env_filter = EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into());

    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_filter(env_filter.clone());

    let Some(log_file) = std::env::var("LOG_FILE")
        .ok()
        .filter(|value| !value.trim().is_empty())
    else {
        tracing_subscriber::registry().with(stderr_layer).init();
        return;
    };

    let Some(file_writer) = FileWriter::new(&log_file) else {
        tracing_subscriber::registry().with(stderr_layer).init();
        return;
    };

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_filter(env_filter);

    tracing_subscriber::registry()
        .with(stderr_layer)
        .with(file_layer)
        .init();
}

#[derive(Clone)]
struct FileWriter {
    file: Arc<Mutex<std::fs::File>>,
}

impl FileWriter {
    fn new(path: &str) -> Option<Self> {
        let path = Path::new(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok()?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .ok()?;

        Some(Self {
            file: Arc::new(Mutex::new(file)),
        })
    }
}

impl<'a> MakeWriter<'a> for FileWriter {
    type Writer = FileWriterGuard;

    fn make_writer(&'a self) -> Self::Writer {
        FileWriterGuard {
            file: Arc::clone(&self.file),
        }
    }
}

struct FileWriterGuard {
    file: Arc<Mutex<std::fs::File>>,
}

impl Write for FileWriterGuard {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut file = self
            .file
            .lock()
            .map_err(|_| io::Error::other("Couldn't lock the log file"))?;
        file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut file = self
            .file
            .lock()
            .map_err(|_| io::Error::other("Couldn't lock the log file"))?;
        file.flush()
    }
}
