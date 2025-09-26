/// Copyright 2025 Miku Push! Team
///
/// Licensed under the Apache License, Version 2.0 (the "License");
/// you may not use this file except in compliance with the License.
/// You may obtain a copy of the License at
///
///     http://www.apache.org/licenses/LICENSE-2.0
///
/// Unless required by applicable law or agreed to in writing, software
/// distributed under the License is distributed on an "AS IS" BASIS,
/// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
/// See the License for the specific language governing permissions and
/// limitations under the License.

use std::fs::File;
use std::path::PathBuf;
use serde::Deserialize;
use tracing::{debug, warn};
use crate::config::{DataBase, LoggingConfig, Server};
use crate::config::upload::Upload;
use crate::logging::local_trace;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub server: Server,
    #[serde(default)]
    pub log: LoggingConfig,
    #[serde(default)]
    pub database: DataBase,
    #[serde(default)]
    pub upload: Upload
}

impl Settings {
    pub fn load() -> Self {
        local_trace(|| {
            Settings::load_from_file()
                .or_else(|| Some(Settings::default()))
                .unwrap()
        })
    }

    fn load_from_file() -> Option<Self> {
        let path = Self::resolve_path();

        if path.is_none() {
            return None;
        }

        let path = path.unwrap();
        let file = match File::open(path.clone()) {
            Err(e) => {
                warn!("failed to open configuration file: {}: {}", path.display(), e);
                return None;
            },
            Ok(file) => file
        };

        match serde_yaml::from_reader(file) {
            Err(e) => {
                warn!("failed to parse configuration file: {}: {}", path.display(), e);
                None
            },
            Ok(settings) => {
                debug!("successfully loaded configuration file: {}", path.display());
                Some(settings)
            }
        }
    }

    fn resolve_path() -> Option<PathBuf> {
        #[cfg(target_os = "linux")]
        let paths: Vec<PathBuf> = vec![
            PathBuf::from("config.yaml"),
            PathBuf::from(format!("{}/.io.mikupush.server/config.yaml", env!("HOME"))),
            PathBuf::from(format!("{}/.config/io.mikupush.server/config.yaml", env!("HOME"))),
            PathBuf::from("/etc/io.mikupush.server/config.yaml"),
        ];

        #[cfg(target_os = "windows")]
        let paths: Vec<PathBuf> = vec![
            PathBuf::from("config.yaml"),
            PathBuf::from(format!("{}\\AppData\\Local\\io.mikupush.server", env!("LOCALAPPDATA"))),
        ];

        #[cfg(target_os = "macos")]
        let paths: Vec<PathBuf> = vec![
            PathBuf::from("config.yaml"),
            PathBuf::from(format!("{}/.io.mikupush.server/config.yaml", env!("HOME"))),
            PathBuf::from(format!("{}/.config/io.mikupush.server/config.yaml", env!("HOME"))),
            PathBuf::from(format!("{}/Library/Application Support/io.mikupush.server/config.yaml", env!("HOME"))),
        ];

        let mut existing_path = None;
        for path in paths {
            debug!("attempting to load configuration file: {}", path.display());

            if !path.exists() {
                debug!("configuration file not found: {}", path.display());
                continue;
            }

            existing_path = Some(path);
        }

        existing_path
    }
}

impl Default for Settings {
    fn default() -> Self {
        local_trace(|| debug!("using default configuration"));

        Settings {
            server: Server::default(),
            log: LoggingConfig::default(),
            database: DataBase::default(),
            upload: Upload::default(),
        }
    }
}
