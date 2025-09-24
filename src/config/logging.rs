use std::fmt::Display;
use tracing::{debug, warn};
use serde::Deserialize;
use tracing::Level;
use crate::config::env;
use crate::logging::{local_trace, system_log_directory};

#[derive(Debug, Deserialize, PartialEq, Clone, Copy)]
pub enum LoggingOutput {
    #[serde(rename = "console")]
    Console,
    #[serde(rename = "file")]
    File,
}

impl Default for LoggingOutput {
    fn default() -> Self {
        LoggingOutput::Console
    }
}

impl Display for LoggingOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoggingOutput::Console => write!(f, "console"),
            LoggingOutput::File => write!(f, "file"),
        }
    }
}

impl LoggingOutput {
    pub fn from_string(value: String) -> LoggingOutput {
        match value.as_str() {
            "console" => LoggingOutput::Console,
            "file" => LoggingOutput::File,
            _ => {
                warn!("log output {} is not supported, using console as default", value);
                LoggingOutput::Console
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum LoggingLevel {
    #[serde(rename = "trace")]
    Trace,
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
}

impl Default for LoggingLevel {
    fn default() -> Self {
        LoggingLevel::Info
    }
}

impl LoggingLevel {
    pub fn from_string(value: String) -> LoggingLevel {
        match value.as_str() {
            "trace" => LoggingLevel::Trace,
            "debug" => LoggingLevel::Debug,
            "info" => LoggingLevel::Info,
            "warn" => LoggingLevel::Warn,
            "error" => LoggingLevel::Error,
            _ => {
                warn!("log level {} is not supported, using {} as default", value, LoggingLevel::default());
                LoggingLevel::default()
            }
        }
    }

    pub fn as_tracing_enum(&self) -> Level {
        match self {
            LoggingLevel::Debug => Level::DEBUG,
            LoggingLevel::Info => Level::INFO,
            LoggingLevel::Warn => Level::WARN,
            LoggingLevel::Error => Level::ERROR,
            LoggingLevel::Trace => Level::TRACE
        }
    }
}

impl Display for LoggingLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoggingLevel::Trace => write!(f, "trace"),
            LoggingLevel::Debug => write!(f, "debug"),
            LoggingLevel::Info => write!(f, "info"),
            LoggingLevel::Warn => write!(f, "warn"),
            LoggingLevel::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    #[serde(default)]
    level: Option<LoggingLevel>,
    #[serde(default)]
    output: Option<LoggingOutput>,
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    directory: Option<String>,
    #[serde(default)]
    json: Option<bool>
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: None,
            output: None,
            file: None,
            directory: None,
            json: None
        }
    }
}

impl LoggingConfig {
    pub fn level(&self) -> LoggingLevel {
        if let Some(value) = env("MIKU_PUSH_LOG_LEVEL") {
            local_trace(|| debug!("using env variable MIKU_PUSH_LOG_LEVEL: {}", value));
            return LoggingLevel::from_string(value);
        }

        let value = self.level.clone();
        if let Some(value) = value {
            local_trace(|| debug!("using log.level configuration: {}", value));
            return value;
        }

        local_trace(|| debug!("using log.level default value: {}", LoggingLevel::default()));
        LoggingLevel::default()
    }

    pub fn output(&self) -> LoggingOutput {
        if let Some(value) = env("MIKU_PUSH_LOG_OUTPUT") {
            local_trace(|| debug!("using env variable MIKU_PUSH_LOG_OUTPUT: {}", value));
            return LoggingOutput::from_string(value);
        }

        let value = self.output.clone();
        if let Some(value) = value {
            local_trace(|| debug!("using log.output configuration: {}", value));
            return value;
        }

        local_trace(|| debug!("using log.output default value: {}", LoggingOutput::default()));
        LoggingOutput::default()
    }

    pub fn file(&self) -> String {
        if let Some(value) = env("MIKU_PUSH_LOG_FILE") {
            local_trace(|| debug!("using env variable MIKU_PUSH_LOG_FILE: {}", value));
            return value;
        }

        let value = self.file.clone();
        if let Some(value) = value {
            local_trace(|| debug!("using log.file configuration: {}", value));
            return value;
        }

        local_trace(|| debug!("using log.file default value: {}", LoggingLevel::default()));
        "server.log".to_string()
    }

    pub fn directory(&self) -> String {
        if let Some(value) = env("MIKU_PUSH_LOG_DIRECTORY") {
            local_trace(|| debug!("using env variable MIKU_PUSH_LOG_DIRECTORY: {}", value));
            return value;
        }

        let value = self.directory.clone();
        if let Some(value) = value {
            local_trace(|| debug!("using log.directory configuration: {}", value));
            return value;
        }

        let directory = system_log_directory();
        local_trace(|| debug!("using log.directory default value: {}", directory));
        directory
    }

    pub fn json(&self) -> bool {
        if let Some(value) = env("MIKU_PUSH_LOG_JSON") {
            local_trace(|| debug!("using env variable MIKU_PUSH_LOG_JSON: {}", value));
            return value.to_lowercase() == "true" || value == "1";
        }

        let value = self.json.clone();
        if let Some(value) = value {
            local_trace(|| debug!("using log.json configuration: {}", value));
            return value;
        }

        local_trace(|| debug!("using log.json default value: {}", false));
        false
    }
}

