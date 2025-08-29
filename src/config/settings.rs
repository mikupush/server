use std::fs::File;
use std::path::PathBuf;
use serde::Deserialize;
use log::{debug, warn};
use crate::config::{DataBase, Server};

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub server: Server,
    #[serde(default)]
    pub database: DataBase,
}

impl Settings {
    pub fn load() -> Self {
        Settings::load_from_file()
            .or(Some(Settings::default()))
            .unwrap()
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

        match serde_json::from_reader(file) {
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
        let mut paths: Vec<PathBuf> = vec![];

        paths.push(
            dirs::home_dir().unwrap()
                .join(".config")
                .join("mikupush-server")
                .join("config.yaml")
        );

        #[cfg(target_os = "linux")]
        paths.push(PathBuf::from("/etc/mikupush-server/config.yaml"));

        paths.push(PathBuf::from("config.yaml"));

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
        debug!("using default configuration");

        Settings {
            server: Server::default(),
            database: DataBase::default(),
        }
    }
}
