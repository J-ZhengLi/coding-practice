use std::fs::File;
use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, EnvFilter, layer::{Layer, SubscriberExt}, util::SubscriberInitExt};
use tracing_subscriber::filter::LevelFilter;

pub fn init() -> WorkerGuard {
    let log_dir = get_log_dir();

    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory {:?}: {}", log_dir, e);
        let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stderr());
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::DEBUG.into())
                    .from_env_lossy(),
            )
            .with_writer(non_blocking)
            .init();
        return guard;
    }

    let log_filename = format!("{}.log", chrono::Local::now().format("%Y-%m-%d_%H-%M-%S"));
    let log_path = log_dir.join(&log_filename);

    let file = match File::create(&log_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create log file {:?}: {}", log_path, e);
            let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stderr());
            tracing_subscriber::fmt()
                .with_env_filter(
                    EnvFilter::builder()
                        .with_default_directive(LevelFilter::DEBUG.into())
                        .from_env_lossy(),
                )
                .with_writer(non_blocking)
                .init();
            return guard;
        }
    };

    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    // Console: Debug+ with ANSI formatting
    let console_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        );

    // File: Info+ without ANSI formatting
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        );

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    tracing::info!("Logging initialized: {}", log_path.display());

    guard
}

fn get_log_dir() -> PathBuf {
    crate::db::get_database_path()
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("logs")
}