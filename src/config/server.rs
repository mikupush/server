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

use crate::config::env;
use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    #[serde(default)]
    host: Option<String>,
    #[serde(default)]
    port: Option<u16>,
    #[serde(default)]
    static_directory: Option<String>,
    #[serde(default)]
    templates_directory: Option<String>,
}

impl Server {
    pub fn host(&self) -> String {
        if let Some(value) = env("MIKU_PUSH_SERVER_HOST") {
            debug!("using env variable MIKU_PUSH_SERVER_HOST: {}", value);
            return value;
        }

        let value = self.host.clone();
        if let Some(value) = value {
            debug!("using server.host configuration: {}", value);
            return value;
        }

        let value = "0.0.0.0".to_string();
        debug!("using server.host default value: {}", value);
        value
    }

    pub fn port(&self) -> u16 {
        if let Some(value) = env("MIKU_PUSH_SERVER_PORT") {
            debug!("using env variable MIKU_PUSH_SERVER_PORT: {}", value);
            return value.parse().expect("Server port must be a number");
        }

        let value = self.port.clone();
        if let Some(value) = value {
            debug!("using server.port configuration: {}", value);
            return value;
        }

        let value = 8080;
        debug!("using server.port default value: {}", value);
        value
    }

    pub fn static_directory(&self) -> String {
        if let Some(value) = env("MIKU_PUSH_SERVER_STATIC_DIR") {
            debug!("using env variable MIKU_PUSH_SERVER_STATIC_DIR: {}", value);
            return value;
        }

        if let Some(value) = self.static_directory.clone() {
            debug!("using server.static_directory configuration: {}", value);
            return value;
        }

        let value = "static".to_string();
        debug!("using server.static_directory default value: {}", value);
        value
    }

    pub fn templates_directory(&self) -> String {
        if let Some(value) = env("MIKU_PUSH_SERVER_TEMPLATES_DIR") {
            debug!("using env variable MIKU_PUSH_SERVER_TEMPLATES_DIR: {}", value);
            return value;
        }

        if let Some(value) = self.templates_directory.clone() {
            debug!("using server.templates_directory configuration: {}", value);
            return value;
        }

        let value = "templates".to_string();
        debug!("using server.templates_directory default value: {}", value);
        value
    }
}

impl Default for Server {
    fn default() -> Self {
        Server {
            host: None,
            port: None,
            static_directory: None,
            templates_directory: None,
        }
    }
}