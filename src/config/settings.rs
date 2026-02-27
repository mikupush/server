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
            host: env.database.host
                .or_else(|| log_yaml_config("database.host", yaml.database.host))
                .or_else(|| log_default_config("database.host", Some(default.host)))
                .unwrap(),
            port: env.database.port
                .or_else(|| log_yaml_config("database.port", yaml.database.port))
                .or_else(|| log_default_config("database.port", Some(default.port)))
                .unwrap(),
            database: env.database.database
                .or_else(|| log_yaml_config("database.database", yaml.database.database))
                .or_else(|| log_default_config("database.database", Some(default.database)))
                .unwrap(),
            user: env.database.user
                .or_else(|| log_yaml_config("database.user", yaml.database.user))
                .or_else(|| log_default_config("database.user", Some(default.user)))
                .unwrap(),
            password: env.database.password
                .or_else(|| log_yaml_config("database.password", yaml.database.password))
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
            level: env.log.level
                .or_else(|| log_yaml_config("log.level", yaml.log.level))
                .or_else(|| log_default_config("log.level", Some(default.level)))
                .unwrap(),
            file_prefix: env.log.file_prefix
                .or_else(|| log_yaml_config("log.file_prefix", yaml.log.file_prefix))
                .or_else(|| log_default_config("log.file_prefix", Some(default.file_prefix)))
                .unwrap(),
            directory: env.log.directory
                .or_else(|| log_yaml_config("log.directory", yaml.log.directory))
                .or_else(|| log_default_config("log.directory", Some(default.directory)))
                .unwrap(),
            json: env.log.json
                .or_else(|| log_yaml_config("log.json", yaml.log.json))
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
    pub static_base_path: String,
    pub templates_directory: String,
}

impl Server {
    pub fn from(yaml: YamlSettings, env: EnvSettings) -> Self {
        let default = Self::default();
        Self {
            host: env.server.host
                .or_else(|| log_yaml_config("server.host", yaml.server.host))
                .or_else(|| log_default_config("server.host", Some(default.host)))
                .unwrap(),
            port: env.server.port
                .or_else(|| log_yaml_config("server.port", yaml.server.port))
                .or_else(|| log_default_config("server.port", Some(default.port)))
                .unwrap(),
            static_directory: env.server.static_directory
                .or_else(|| log_yaml_config("server.static_directory", yaml.server.static_directory))
                .or_else(|| log_default_config("server.static_directory", Some(default.static_directory)))
                .unwrap(),
            static_base_path: env.server.static_base_path
                .or_else(|| log_yaml_config("server.static_base_path", yaml.server.static_base_path))
                .or_else(|| log_default_config("server.static_base_path", Some(default.static_base_path)))
                .unwrap(),
            templates_directory: env.server.templates_directory
                .or_else(|| log_yaml_config("server.templates_directory", yaml.server.templates_directory))
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
            static_directory: "dist/assets".to_string(),
            static_base_path: "/assets".to_string(),
            templates_directory: "dist".to_string(),
        }
    }

}

#[derive(Debug, Clone)]
pub struct Upload {
    pub max_size: Option<u64>,
    pub directory: String,
    pub expires_in_seconds: Option<u64>,
    pub expiration_cleanup_interval_seconds: u64,
}

impl Upload {
    pub fn from(yaml: YamlSettings, env: EnvSettings) -> Self {
        let default = Self::default();
        Self {
            max_size: env.upload.max_size
                .or_else(|| log_yaml_config("upload.max_size", yaml.upload.max_size))
                .or_else(|| log_default_config("upload.max_size", default.max_size)),
            directory: env.upload.directory
                .or_else(|| log_yaml_config("upload.directory", yaml.upload.directory))
                .or_else(|| log_default_config("upload.directory", Some(default.directory)))
                .unwrap(),
            expires_in_seconds: env.upload.expires_in_seconds
                .or_else(|| log_yaml_config("upload.expires_in_seconds", yaml.upload.expires_in_seconds))
                .or_else(|| log_default_config("upload.expires_in_seconds", default.expires_in_seconds)),
            expiration_cleanup_interval_seconds: env.upload.expiration_cleanup_interval_seconds
                .or_else(|| log_yaml_config("upload.expiration_cleanup_interval_seconds", yaml.upload.expiration_cleanup_interval_seconds))
                .or_else(|| log_default_config("upload.expiration_cleanup_interval_seconds", Some(default.expiration_cleanup_interval_seconds)))
                .unwrap(),
        }
    }

    pub fn create_with_limit(limit: u64) -> Self {
        let default = Self::default();

        Self {
            max_size: Some(limit),
            directory: default.directory,
            expires_in_seconds: default.expires_in_seconds,
            expiration_cleanup_interval_seconds: default.expiration_cleanup_interval_seconds,
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
            expires_in_seconds: None,
            expiration_cleanup_interval_seconds: 3600,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Debug {
    pub enable: bool,
    pub astro_dev_server: String,
}

impl Debug {
    pub fn from(yaml: YamlSettings, env: EnvSettings) -> Self {
        let default = Self::default();
        Self {
            enable: env.debug.enable
                .or_else(|| log_yaml_config("debug.enable", yaml.debug.enable))
                .or_else(|| log_default_config("debug.enable", Some(default.enable)))
                .unwrap(),
            astro_dev_server: env.debug.astro_dev_server
                .or_else(|| log_yaml_config("debug.astro_dev_server", yaml.debug.astro_dev_server))
                .or_else(|| log_default_config("debug.astro_dev_server", Some(default.astro_dev_server)))
                .unwrap(),
        }
    }
}

impl Default for Debug {
    fn default() -> Self {
        Self {
            enable: false,
            astro_dev_server: "http://localhost:4321/".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub server: Server,
    pub log: LoggingConfig,
    pub database: DataBase,
    pub upload: Upload,
    pub debug: Debug,
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
    pub fn new(server: Server, log: LoggingConfig, database: DataBase, upload: Upload, debug: Debug) -> Self {
        Self { server, log, database, upload, debug }
    }

    pub fn from(yaml: YamlSettings, env: EnvSettings) -> Self {
        Self {
            server: Server::from(yaml.clone(), env.clone()),
            log: LoggingConfig::from(yaml.clone(), env.clone()),
            database: DataBase::from(yaml.clone(), env.clone()),
            upload: Upload::from(yaml.clone(), env.clone()),
            debug: Debug::from(yaml, env),
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
            debug: Debug::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::env::{EnvDataBase, EnvDebug, EnvLoggingConfig, EnvServer, EnvUpload};
    use crate::config::yaml::YamlSettings;

    #[test]
    fn test_server_config_precedence() {
        // 1. Env > Yaml
        let mut env = EnvSettings {
            server: EnvServer::default(),
            log: EnvLoggingConfig::default(),
            database: EnvDataBase::default(),
            upload: EnvUpload::default(),
            debug: EnvDebug::default(),
        };
        env.server.host = Some("env-host".to_string());
        env.server.port = Some(9090);
        env.server.static_base_path = Some("/env-assets".to_string());

        let mut yaml = YamlSettings::default();
        yaml.server.host = Some("yaml-host".to_string());
        yaml.server.port = Some(8080);
        yaml.server.static_base_path = Some("/yaml-assets".to_string());

        let server = Server::from(yaml.clone(), env.clone());
        assert_eq!(server.host, "env-host");
        assert_eq!(server.port, 9090);
        assert_eq!(server.static_base_path, "/env-assets");

        // 2. Yaml > Default (Env is None)
        let env_empty = EnvSettings {
            server: EnvServer::default(),
            log: EnvLoggingConfig::default(),
            database: EnvDataBase::default(),
            upload: EnvUpload::default(),
            debug: EnvDebug::default(),
        };

        let server = Server::from(yaml.clone(), env_empty.clone());
        assert_eq!(server.host, "yaml-host");
        assert_eq!(server.port, 8080);
        assert_eq!(server.static_base_path, "/yaml-assets");

        // 3. Default (Env and Yaml are None)
        let yaml_empty = YamlSettings::default();
        let server = Server::from(yaml_empty, env_empty);
        assert_eq!(server.host, "0.0.0.0");
        assert_eq!(server.port, 8080);
        assert_eq!(server.static_base_path, "/assets");
    }

    #[test]
    fn test_database_config_precedence() {
        let mut env = EnvSettings {
            server: EnvServer::default(),
            log: EnvLoggingConfig::default(),
            database: EnvDataBase::default(),
            upload: EnvUpload::default(),
            debug: EnvDebug::default(),
        };
        env.database.host = Some("env-db-host".to_string());
        
        let mut yaml = YamlSettings::default();
        yaml.database.host = Some("yaml-db-host".to_string());

        // Env > Yaml
        let db = DataBase::from(yaml.clone(), env.clone());
        assert_eq!(db.host, "env-db-host");

        // Yaml > Default
        let env_empty = EnvSettings {
            server: EnvServer::default(),
            log: EnvLoggingConfig::default(),
            database: EnvDataBase::default(),
            upload: EnvUpload::default(),
            debug: EnvDebug::default(),
        };
        let db = DataBase::from(yaml.clone(), env_empty.clone());
        assert_eq!(db.host, "yaml-db-host");

        // Default
        let yaml_empty = YamlSettings::default();
        let db = DataBase::from(yaml_empty, env_empty);
        assert_eq!(db.host, "localhost");
    }

    #[test]
    fn test_logging_config_precedence() {
        let mut env = EnvSettings {
            server: EnvServer::default(),
            log: EnvLoggingConfig::default(),
            database: EnvDataBase::default(),
            upload: EnvUpload::default(),
            debug: EnvDebug::default(),
        };
        env.log.file_prefix = Some("env-prefix".to_string());
        env.log.json = Some(true);

        let mut yaml = YamlSettings::default();
        yaml.log.file_prefix = Some("yaml-prefix".to_string());
        yaml.log.json = Some(false);

        // Env > Yaml
        let log = LoggingConfig::from(yaml.clone(), env.clone());
        assert_eq!(log.file_prefix, "env-prefix");
        assert_eq!(log.json, true);

        // Yaml > Default
        let env_empty = EnvSettings {
            server: EnvServer::default(),
            log: EnvLoggingConfig::default(),
            database: EnvDataBase::default(),
            upload: EnvUpload::default(),
            debug: EnvDebug::default(),
        };
        let log = LoggingConfig::from(yaml.clone(), env_empty.clone());
        assert_eq!(log.file_prefix, "yaml-prefix");
        assert_eq!(log.json, false);

        // Default
        let yaml_empty = YamlSettings::default();
        let log = LoggingConfig::from(yaml_empty, env_empty);
        assert_eq!(log.file_prefix, "server");
        assert_eq!(log.json, false);
    }

    #[test]
    fn test_upload_config_precedence() {
        let mut env = EnvSettings {
            server: EnvServer::default(),
            log: EnvLoggingConfig::default(),
            database: EnvDataBase::default(),
            upload: EnvUpload::default(),
            debug: EnvDebug::default(),
        };
        env.upload.directory = Some("env-data".to_string());
        env.upload.max_size = Some(100);

        let mut yaml = YamlSettings::default();
        yaml.upload.directory = Some("yaml-data".to_string());
        yaml.upload.max_size = Some(200);

        // Env > Yaml
        let upload = Upload::from(yaml.clone(), env.clone());
        assert_eq!(upload.directory, "env-data");
        assert_eq!(upload.max_size, Some(100));

        // Yaml > Default
        let env_empty = EnvSettings {
            server: EnvServer::default(),
            log: EnvLoggingConfig::default(),
            database: EnvDataBase::default(),
            upload: EnvUpload::default(),
            debug: EnvDebug::default(),
        };
        let upload = Upload::from(yaml.clone(), env_empty.clone());
        assert_eq!(upload.directory, "yaml-data");
        assert_eq!(upload.max_size, Some(200));

        // Default
        let yaml_empty = YamlSettings::default();
        let upload = Upload::from(yaml_empty, env_empty);
        assert_eq!(upload.directory, "data");
        assert_eq!(upload.max_size, None);
    }

    #[test]
    fn test_debug_config_precedence() {
        let mut env = EnvSettings {
            server: EnvServer::default(),
            log: EnvLoggingConfig::default(),
            database: EnvDataBase::default(),
            upload: EnvUpload::default(),
            debug: EnvDebug::default(),
        };
        env.debug.enable = Some(true);
        env.debug.astro_dev_server = Some("http://env-astro".to_string());

        let mut yaml = YamlSettings::default();
        yaml.debug.enable = Some(false);
        yaml.debug.astro_dev_server = Some("http://yaml-astro".to_string());

        // Env > Yaml
        let debug = Debug::from(yaml.clone(), env.clone());
        assert_eq!(debug.enable, true);
        assert_eq!(debug.astro_dev_server, "http://env-astro");

        // Yaml > Default
        let env_empty = EnvSettings {
            server: EnvServer::default(),
            log: EnvLoggingConfig::default(),
            database: EnvDataBase::default(),
            upload: EnvUpload::default(),
            debug: EnvDebug::default(),
        };
        let debug = Debug::from(yaml.clone(), env_empty.clone());
        assert_eq!(debug.enable, false);
        assert_eq!(debug.astro_dev_server, "http://yaml-astro");

        // Default
        let yaml_empty = YamlSettings::default();
        let debug = Debug::from(yaml_empty, env_empty);
        assert_eq!(debug.enable, false);
        assert_eq!(debug.astro_dev_server, "http://localhost:4321/");
    }
}
