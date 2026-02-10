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

use crate::config::env::EnvSettings;
use crate::config::yaml::YamlSettings;
use crate::config::{user_config_path, LoggingLevel};
use crate::logging::system_log_directory;
use std::path::PathBuf;
use std::sync::OnceLock;

static SETTINGS_INSTANCE: OnceLock<Settings> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct DataBase {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
}

impl DataBase {
    pub fn from(yaml: YamlSettings, env: EnvSettings) -> Self {
        let default = Self::default();

        Self {
            host: log_yaml_config("database.host", yaml.database.host)
                .or_else(|| env.database.host)
                .or_else(|| log_default_config("database.host", Some(default.host)))
                .unwrap(),
            port: log_yaml_config("database.port", yaml.database.port)
                .or_else(|| env.database.port)
                .or_else(|| log_default_config("database.port", Some(default.port)))
                .unwrap(),
            database: log_yaml_config("database.database", yaml.database.database)
                .or_else(|| env.database.database)
                .or_else(|| log_default_config("database.database", Some(default.database)))
                .unwrap(),
            user: log_yaml_config("database.user", yaml.database.user)
                .or_else(|| env.database.user)
                .or_else(|| log_default_config("database.user", Some(default.user)))
                .unwrap(),
            password: log_yaml_config("database.password", yaml.database.password)
                .or_else(|| env.database.password)
                .or_else(|| log_default_config("database.password", Some(default.password)))
                .unwrap(),
        }
    }
    
    pub fn url(&self) -> String {
        format!("postgresql://{}:{}@{}:{}/{}", self.user, self.password, self.host, self.port, self.database)
    }
}

impl Default for DataBase {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "postgres".to_string(),
            user: "postgres".to_string(),
            password: "postgres".to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: LoggingLevel,
    pub file_prefix: String,
    pub directory: String,
    pub json: bool
}

impl LoggingConfig {
    pub fn from(yaml: YamlSettings, env: EnvSettings) -> Self {
        let default = Self::default();
        Self {
            level: log_yaml_config("log.level", yaml.log.level)
                .or_else(|| env.log.level)
                .or_else(|| log_default_config("log.level", Some(default.level)))
                .unwrap(),
            file_prefix: log_yaml_config("log.file_prefix", yaml.log.file_prefix)
                .or_else(|| env.log.file_prefix)
                .or_else(|| log_default_config("log.file_prefix", Some(default.file_prefix)))
                .unwrap(),
            directory: log_yaml_config("log.directory", yaml.log.directory)
                .or_else(|| env.log.directory)
                .or_else(|| log_default_config("log.directory", Some(default.directory)))
                .unwrap(),
            json: log_yaml_config("log.json", yaml.log.json)
                .or_else(|| env.log.json)
                .or_else(|| log_default_config("log.json", Some(default.json)))
                .unwrap(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LoggingLevel::default(),
            file_prefix: "server".to_string(),
            directory: system_log_directory(),
            json: false
        }
    }
}

#[derive(Debug, Clone)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub static_directory: String,
    pub templates_directory: String,
}

impl Server {
    pub fn from(yaml: YamlSettings, env: EnvSettings) -> Self {
        let default = Self::default();
        Self {
            host: log_yaml_config("server.host", yaml.server.host)
                .or_else(|| env.server.host)
                .or_else(|| log_default_config("server.host", Some(default.host)))
                .unwrap(),
            port: log_yaml_config("server.port", yaml.server.port)
                .or_else(|| env.server.port)
                .or_else(|| log_default_config("server.port", Some(default.port)))
                .unwrap(),
            static_directory: log_yaml_config("server.static_directory", yaml.server.static_directory)
                .or_else(|| env.server.static_directory)
                .or_else(|| log_default_config("server.static_directory", Some(default.static_directory)))
                .unwrap(),
            templates_directory: log_yaml_config("server.templates_directory", yaml.server.templates_directory)
                .or_else(|| env.server.templates_directory)
                .or_else(|| log_default_config("server.templates_directory", Some(default.templates_directory)))
                .unwrap(),
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            static_directory: "static".to_string(),
            templates_directory: "templates".to_string(),
        }
    }

}

#[derive(Debug, Clone)]
pub struct Upload {
    pub max_size: Option<u64>,
    pub directory: String,
    pub expires_in_days: Option<u64>,
}

impl Upload {
    pub fn from(yaml: YamlSettings, env: EnvSettings) -> Self {
        let default = Self::default();
        Self {
            max_size: log_yaml_config("upload.max_size", yaml.upload.max_size)
                .or_else(|| env.upload.max_size)
                .or_else(|| log_default_config("upload.max_size", default.max_size)),
            directory: log_yaml_config("upload.directory", yaml.upload.directory)
                .or_else(|| env.upload.directory)
                .or_else(|| log_default_config("upload.directory", Some(default.directory)))
                .unwrap(),
            expires_in_days: log_yaml_config("upload.expires_in_days", yaml.upload.expires_in_days)
                .or_else(|| env.upload.expires_in_days)
                .or_else(|| log_default_config("upload.expires_in_days", default.expires_in_days)),
        }
    }

    pub fn create_with_limit(limit: u64) -> Self {
        let default = Self::default();

        Self {
            max_size: Some(limit),
            directory: default.directory,
            expires_in_days: default.expires_in_days,
        }
    }

    pub fn is_limited(&self) -> bool {
        self.max_size.is_some()
    }
}

impl Default for Upload {
    fn default() -> Self {
        Self {
            max_size: None,
            directory: "data".to_string(),
            expires_in_days: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub server: Server,
    pub log: LoggingConfig,
    pub database: DataBase,
    pub upload: Upload
}

fn log_yaml_config<T>(key: &str, value: Option<T>) -> Option<T>
where
    T: std::fmt::Debug
{
    if value.is_some() {
        println!("using yaml config {}: {:?}", key, value);
    }

    value
}

fn log_default_config<T>(key: &str, value: Option<T>) -> Option<T>
where
    T: std::fmt::Debug
{
    if value.is_some() {
        println!("using default config {}: {:?}", key, value);
    }

    value
}

impl Settings {
    pub fn new(server: Server, log: LoggingConfig, database: DataBase, upload: Upload) -> Self {
        Self { server, log, database, upload }
    }

    pub fn from(yaml: YamlSettings, env: EnvSettings) -> Self {
        Self {
            server: Server::from(yaml.clone(), env.clone()),
            log: LoggingConfig::from(yaml.clone(), env.clone()),
            database: DataBase::from(yaml.clone(), env.clone()),
            upload: Upload::from(yaml, env),
        }
    }

    pub fn setup_global_from(path: Option<PathBuf>) -> Self {
        Self::check_specified_path_exists(path.clone());
        let settings = Settings::load(path);
        let result = SETTINGS_INSTANCE.set(settings.clone());

        if let Err(_) = result {
            println!("failed to set global configuration, it could be already set");
        }

        settings
    }

    pub fn get() -> Settings {
        if let Some(settings) = SETTINGS_INSTANCE.get() {
            return settings.clone();
        }

        panic!("failed to get global settings, it could be setup before");
    }

    pub fn load(path: Option<PathBuf>) -> Self {
        let path = path.unwrap_or_else(|| user_config_path());
        let yaml = YamlSettings::load(path);
        let env = EnvSettings::load();
        Settings::from(yaml, env)
    }

    fn check_specified_path_exists(path: Option<PathBuf>) {
        if let Some(path) = path && !path.exists() {
            panic!(
                "error: configuration file not found: {}\n\
                 Use -c <path> or --config <path> with an existing file.",
                path.display()
            );
        };
    }
}

impl Default for Settings {
    fn default() -> Self {
        println!("using default configuration");

        Settings {
            server: Server::default(),
            log: LoggingConfig::default(),
            database: DataBase::default(),
            upload: Upload::default(),
        }
    }
}
