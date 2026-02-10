use crate::config::LoggingLevel;
use std::collections::{HashMap, VecDeque};
use std::sync::LazyLock;
use futures::future::Lazy;

#[derive(Debug, Clone)]
pub struct EnvDataBase {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
}

impl EnvDataBase {
    pub fn load() -> Self {
        Self {
            host: env("MIKU_PUSH_DATABASE_HOST"),
            port: env("MIKU_PUSH_DATABASE_PORT")
                .map(|value| value.parse().expect("Database port must be a number")),
            database: env("MIKU_PUSH_DATABASE_NAME"),
            user: env("MIKU_PUSH_DATABASE_USER"),
            password: env("MIKU_PUSH_DATABASE_PASSWORD"),
        }
    }
}

impl Default for EnvDataBase {
    fn default() -> Self {
        Self {
            host: None,
            port: None,
            database: None,
            user: None,
            password: None
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvLoggingConfig {
    pub level: Option<LoggingLevel>,
    pub file_prefix: Option<String>,
    pub directory: Option<String>,
    pub json: Option<bool>
}

impl EnvLoggingConfig {
    pub fn load() -> Self {
        Self {
            level: env("MIKU_PUSH_LOG_LEVEL")
                .map(|value| LoggingLevel::from_string(value)),
            file_prefix: env("MIKU_PUSH_LOG_FILE_PREFIX"),
            directory: env("MIKU_PUSH_LOG_DIRECTORY"),
            json: env("MIKU_PUSH_LOG_JSON")
                .map(|value| value.to_lowercase() == "true" || value == "1"),
        }
    }
}

impl Default for EnvLoggingConfig {
    fn default() -> Self {
        Self {
            level: None,
            file_prefix: None,
            directory: None,
            json: None
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvServer {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub static_directory: Option<String>,
    pub templates_directory: Option<String>,
}

impl EnvServer {
    pub fn load() -> Self {
        Self {
            host: env("MIKU_PUSH_SERVER_HOST"),
            port: env("MIKU_PUSH_SERVER_PORT")
                .map(|value| value.parse().expect("Server port must be a number")),
            static_directory: env("MIKU_PUSH_SERVER_STATIC_DIR"),
            templates_directory: env("MIKU_PUSH_SERVER_TEMPLATES_DIR"),
        }
    }
}

impl Default for EnvServer {
    fn default() -> Self {
        Self {
            host: None,
            port: None,
            static_directory: None,
            templates_directory: None
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvUpload {
    pub max_size: Option<u64>,
    pub directory: Option<String>,
    pub expires_in_days: Option<u64>,
    pub expiration_cleanup_interval_seconds: Option<u64>,
}

impl EnvUpload {
    pub fn load() -> Self {
        Self {
            max_size: env("MIKU_PUSH_UPLOAD_MAX_SIZE")
                .map(|value| value.parse().expect("upload max size must be a number")),
            directory: env("MIKU_PUSH_UPLOAD_DIRECTORY"),
            expires_in_days: env("MIKU_PUSH_UPLOAD_EXPIRES_IN_DAYS")
                .map(|value| value.parse().expect("upload expiration must be a number")),
            expiration_cleanup_interval_seconds: env("MIKU_PUSH_UPLOAD_EXPIRATION_CLEANUP_INTERVAL")
                .map(|value| value.parse().expect("upload expiration cleanup interval must be a number")),
        }
    }
}

impl Default for EnvUpload {
    fn default() -> Self {
        Self { max_size: None, directory: None, expires_in_days: None, expiration_cleanup_interval_seconds: None }
    }
}

#[derive(Debug, Clone)]
pub struct EnvSettings {
    pub server: EnvServer,
    pub log: EnvLoggingConfig,
    pub database: EnvDataBase,
    pub upload: EnvUpload
}

impl EnvSettings {
    pub fn load() -> Self {
        Self {
            server: EnvServer::load(),
            log: EnvLoggingConfig::load(),
            database: EnvDataBase::load(),
            upload: EnvUpload::load()
        }
    }
}

static DOTENV_VARS: LazyLock<HashMap<String, String>> = LazyLock::new(|| load_dotenv());

fn env(name: &str) -> Option<String> {
    if let Some(value) = DOTENV_VARS.get(name) {
        println!("using dotenv variable {}: {}", name, value);
        return Some(value.to_string());
    }

    if let Some(value) = std::env::var(name).ok() {
        println!("using env variable {}: {}", name, value);
        return Some(value.to_string());
    }

    None
}

fn load_dotenv() -> HashMap<String, String> {
    let mut env_files: VecDeque<&str> = VecDeque::new();
    env_files.push_back(".env");

    #[cfg(test)]
    env_files.push_back(".env.test");

    for env_file in env_files {
        println!("loading dotenv file: {}", env_file);

        let dotenv_variables = dotenvy::from_filename_iter(env_file);
        if let Err(err) = dotenv_variables {
            println!("failed to load dotenv file {}: {}", env_file, err);
            continue;
        }

        println!("dotenv file {} loaded!", env_file);
        let mut variables = HashMap::new();
        for item in dotenv_variables.unwrap() {
            if let Ok((key, value)) = item {
                variables.insert(key, value);
            }
        }

        return variables
    }

    HashMap::new()
}
