// Miku Push! Server is the backend behind Miku Push!
// Copyright (C) 2025  Miku Push! Team
// 
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::config::Settings;
use std::path::Path;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{prelude::*, registry};

pub fn configure_logging(settings: &Settings) -> WorkerGuard {
    let config = settings.log.clone();
    let directory = config.directory;
    let directory = Path::new(&directory);
    if !directory.exists() {
        std::fs::create_dir(directory).expect("failed to create log directory");
    }

    let file_appender = Builder::new()
        .rotation(Rotation::HOURLY)
        .filename_prefix(config.file_prefix)
        .filename_suffix("log")
        .build(directory)
        .expect("failed to create log file appender");

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let level_filter = LevelFilter::from_level(config.level.as_tracing_enum());

    if config.json {
        let file_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_level(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(non_blocking)
            .json()
            .flatten_event(true);

        let stdout_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_level(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(std::io::stdout)
            .json()
            .flatten_event(true);

        registry()
            .with(level_filter)
            .with(file_layer)
            .with(stdout_layer)
            .init();
    } else {
        let file_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_level(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(non_blocking);

        let stdout_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_level(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(std::io::stdout);

        registry()
            .with(level_filter)
            .with(file_layer)
            .with(stdout_layer)
            .init();
    }

    _guard
}

pub fn system_log_directory() -> String {
    #[cfg(target_os = "windows")]
    let directory = format!("{}\\io.mikupush.server\\logs", env!("LOCALAPPDATA"));

    #[cfg(target_os = "macos")]
    let directory = format!("{}/Library/Logs/io.mikupush.server", env!("HOME"));

    #[cfg(target_os = "linux")]
    let directory = "/var/log/io.mikupush.server".to_string();

    directory
}
