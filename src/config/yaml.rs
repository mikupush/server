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

use crate::config::LoggingLevel;
use serde::Deserialize;
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct YamlDataBase {
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub database: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

impl Default for YamlDataBase {
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

#[derive(Debug, Deserialize, Clone)]
pub struct YamlLoggingConfig {
    #[serde(default)]
    pub level: Option<LoggingLevel>,
    #[serde(default)]
    pub file_prefix: Option<String>,
    #[serde(default)]
    pub directory: Option<String>,
    #[serde(default)]
    pub json: Option<bool>
}

impl Default for YamlLoggingConfig {
    fn default() -> Self {
        Self {
            level: None,
            file_prefix: None,
            directory: None,
            json: None
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct YamlServer {
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub static_directory: Option<String>,
    #[serde(default)]
    pub static_base_path: Option<String>,
    #[serde(default)]
    pub templates_directory: Option<String>,
}

impl Default for YamlServer {
    fn default() -> Self {
        Self {
            host: None,
            port: None,
            static_directory: None,
            static_base_path: None,
            templates_directory: None
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct YamlUpload {
    pub max_size: Option<u64>,
    pub directory: Option<String>,
    pub expires_in_seconds: Option<u64>,
    pub expiration_cleanup_interval_seconds: Option<u64>,
}

impl Default for YamlUpload {
    fn default() -> Self {
        Self { max_size: None, directory: None, expires_in_seconds: None, expiration_cleanup_interval_seconds: None }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct YamlDebug {
    #[serde(default)]
    pub enable: Option<bool>,
    #[serde(default)]
    pub astro_dev_server: Option<String>,
}

impl Default for YamlDebug {
    fn default() -> Self {
        Self {
            enable: None,
            astro_dev_server: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct YamlSettings {
    #[serde(default)]
    pub server: YamlServer,
    #[serde(default)]
    pub log: YamlLoggingConfig,
    #[serde(default)]
    pub database: YamlDataBase,
    #[serde(default)]
    pub upload: YamlUpload,
    #[serde(default)]
    pub debug: YamlDebug,
}

impl YamlSettings {
    pub fn load(path: PathBuf) -> Self {
        println!("reading configuration file: {}", path.display());
        let file = match File::open(path.clone()) {
            Err(e) => {
                println!("failed to open configuration file: {}: {}", path.display(), e);
                return Self::default();
            },
            Ok(file) => file,
        };

        match serde_yaml::from_reader(file) {
            Err(e) => {
                println!("failed to parse configuration file: {}: {}", path.display(), e);
                Self::default()
            },
            Ok(settings) => {
                println!("successfully loaded configuration file: {}", path.display());
                settings
            }
        }
    }
}

impl Default for YamlSettings {
    fn default() -> Self {
        Self {
            server: YamlServer::default(),
            log: YamlLoggingConfig::default(),
            database: YamlDataBase::default(),
            upload: YamlUpload::default(),
            debug: YamlDebug::default(),
        }
    }
}