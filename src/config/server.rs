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

use tracing::debug;
use serde::Deserialize;
use crate::config::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    #[serde(default)]
    host: Option<String>,
    #[serde(default)]
    port: Option<u16>,
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
}

impl Default for Server {
    fn default() -> Self {
        Server {
            host: None,
            port: None,
        }
    }
}
