use std::path::Path;
use tracing::{debug, warn};
use tracing::{Dispatch, Level};
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::NonBlocking;
use crate::config::{LoggingConfig, LoggingOutput, Settings};

pub fn configure_logging(settings: Settings) {
    let config = settings.log;
    match config.output() {
        LoggingOutput::Console => configure_logging_console_output(&config),
        LoggingOutput::File => configure_logging_file_output(&config)
    }
}

fn configure_logging_console_output(config: &LoggingConfig) {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());

    configure_logging_subscriber(&config, non_blocking);
    std::mem::forget(_guard);
}

fn configure_logging_file_output(config: &LoggingConfig) {
    let directory = config.directory();
    let path = Path::new(&directory);
    if !path.exists() {
        local_trace(|| debug!("creating log directory: {}", directory));

        if let Err(err) = std::fs::create_dir(path) {
            local_trace(|| warn!("failed to create log directory: {}", err));
            return
        }
    }

    let file_appender = tracing_appender::rolling::hourly(directory, config.file());
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    configure_logging_subscriber(&config, non_blocking);
    std::mem::forget(_guard);
}

fn configure_logging_subscriber(config: &LoggingConfig, writer: NonBlocking) {
    if config.json() {
        tracing_subscriber::fmt()
            .json()
            .with_ansi(false)
            .with_max_level(LevelFilter::from_level(config.level().as_tracing_enum()))
            .with_level(true)
            .with_writer(writer)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_ansi(false)
            .with_max_level(LevelFilter::from_level(config.level().as_tracing_enum()))
            .with_level(true)
            .with_writer(writer)
            .init();
    };
}

pub fn system_log_directory() -> String {
    #[cfg(target_os = "windows")]
    let directory = format!("{}\\AppData\\Local\\io.mikupush.server", env!("LOCALAPPDATA"));

    #[cfg(target_os = "macos")]
    let directory = format!("{}/Library/Logs/io.mikupush.server", env!("HOME"));

    #[cfg(target_os = "linux")]
    let directory = "/var/log/io.mikupush.server".to_string();

    directory
}

pub fn local_trace<T>(f: impl FnOnce() -> T) -> T {
    let directory = system_log_directory();
    let directory = Path::new(&directory);
    if !directory.exists() {
        std::fs::create_dir(directory).unwrap();
    }

    let file_appender = tracing_appender::rolling::hourly(directory, "server.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_writer(std::io::stdout)
        .with_max_level(Level::DEBUG)
        .finish();
    let dispatch = Dispatch::new(subscriber);
    tracing::dispatcher::with_default(&dispatch, f)
}
