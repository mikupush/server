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

use log::debug;
use serde::Deserialize;
use crate::config::env;

#[derive(Debug, Clone, Deserialize)]
pub struct DataBase {
    #[serde(default)]
    host: Option<String>,
    #[serde(default)]
    port: Option<u16>,
    #[serde(default)]
    database: Option<String>,
    #[serde(default)]
    user: Option<String>,
    #[serde(default)]
    password: Option<String>,
}

impl DataBase {
    pub fn url(&self) -> String {
        let url = format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.user(),
            self.password(),
            self.host(),
            self.port(),
            self.database()
        );

        debug!("created database url from configuration: {}", url);
        url
    }

    pub fn host(&self) -> String {
        if let Some(value) = env("MIKU_PUSH_DATABASE_HOST") {
            debug!("using env variable MIKU_PUSH_DATABASE_HOST: {}", value);
            return value;
        }

        let value = self.host.clone();
        if let Some(value) = value {
            debug!("using database.host configuration: {}", value);
            return value;
        }

        let value = "localhost".to_string();
        debug!("using database.host default value: {}", value);
        value
    }

    pub fn port(&self) -> u16 {
        if let Some(value) = env("MIKU_PUSH_DATABASE_PORT") {
            debug!("using env variable MIKU_PUSH_DATABASE_PORT: {}", value);
            return value.parse().expect("Database port must be a number");
        }

        let value = self.port.clone();
        if let Some(value) = value {
            debug!("using database.port configuration: {}", value);
            return value;
        }

        let value = 5432;
        debug!("using database.port default value: {}", value);
        value
    }

    pub fn database(&self) -> String {
        if let Some(value) = env("MIKU_PUSH_DATABASE_NAME") {
            debug!("using env variable MIKU_PUSH_DATABASE_NAME: {}", value);
            return value;
        }

        let value = self.database.clone();
        if let Some(value) = value {
            debug!("using database.database configuration: {}", value);
            return value;
        }

        let value = "postgres".to_string();
        debug!("using database.database default value: {}", value);
        value
    }

    pub fn user(&self) -> String {
        if let Some(value) = env("MIKU_PUSH_DATABASE_USER") {
            debug!("using env variable MIKU_PUSH_DATABASE_USER: {}", value);
            return value;
        }

        let value = self.user.clone();
        if let Some(value) = value {
            debug!("using database.user configuration: {}", value);
            return value;
        }

        let value = "postgres".to_string();
        debug!("using database.user default value: {}", value);
        value
    }

    pub fn password(&self) -> String {
        if let Some(value) = env("MIKU_PUSH_DATABASE_PASSWORD") {
            debug!("using env variable MIKU_PUSH_DATABASE_PASSWORD: {}", value);
            return value;
        }

        let value = self.password.clone();
        if let Some(value) = value {
            debug!("using database.password configuration: {}", value);
            return value;
        }

        let value = "postgres".to_string();
        debug!("using database.password default value: {}", value);
        value
    }
}

impl Default for DataBase {
    fn default() -> Self {
        DataBase {
            host: None,
            port: None,
            database: None,
            user: None,
            password: None,
        }
    }
}
