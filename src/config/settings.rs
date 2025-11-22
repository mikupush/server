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

use crate::config::upload::Upload;
use crate::config::{DataBase, LoggingConfig, Server};
use crate::logging::local_trace;
use serde::Deserialize;
use std::fs::File;
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing::{debug, warn};

static SETTINGS_INSTANCE: OnceLock<Settings> = OnceLock::new();

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
    pub fn new(server: Server, log: LoggingConfig, database: DataBase, upload: Upload) -> Self {
        Self { server, log, database, upload }
    }

    pub fn setup_global_from(path: Option<PathBuf>) {
        let settings = if let Some(path) = path {
            if !path.exists() {
                panic!(
                    "error: configuration file not found: {}\n\
                    Use -c <path> or --config <path> with an existing file.",
                    path.display()
                );
            }

            Settings::load_from_path(path).expect("failed to load configuration file")
        } else {
            Settings::load()
        };

        let result = SETTINGS_INSTANCE.set(settings);

        if let Err(_) = result {
            let error = "failed to set global configuration, it could be already set";
            local_trace(|| warn!(error));
            warn!(error);
        }
    }

    pub fn get() -> Settings {
        if let Some(settings) = SETTINGS_INSTANCE.get() {
            return settings.clone();
        }

        panic!("failed to get global settings, it could be setup before");
    }

    pub fn load() -> Self {
        local_trace(|| {
            Settings::load_from_file()
                .or_else(|| Some(Settings::default()))
                .unwrap()
        })
    }

    pub fn load_from_path(path: PathBuf) -> Option<Self> {
        debug!("attempting to load configuration file: {}", path.display());
        let file = match File::open(path.clone()) {
            Err(e) => {
                warn!("failed to open configuration file: {}: {}", path.display(), e);
                return None;
            },
            Ok(file) => file,
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

    fn load_from_file() -> Option<Self> {
        let path = Self::resolve_path();

        if path.is_none() {
            return None;
        }

        let path = path.unwrap();
        Self::load_from_path(path)
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
            PathBuf::from(format!("{}\\io.mikupush.server\\config.yaml", env!("LOCALAPPDATA"))),
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
