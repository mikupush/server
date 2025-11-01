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

use tracing::debug;
use serde::{Deserialize, Serialize};
use crate::config::env;

pub const UPLOAD_MAX_SIZE_UNLIMITED: &str = "unlimited";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upload {
    max_size: Option<u64>,
    directory: Option<String>,
}

impl Upload {
    /// Returns true if the upload is limited.
    /// If true, you can unwrap securely the max_size optional
    pub fn is_limited(&self) -> bool {
        self.max_size().is_some()
    }

    pub fn max_size(&self) -> Option<u64> {
        if let Some(value) = env("MIKU_PUSH_UPLOAD_MAX_SIZE") {
            debug!("using env variable MIKU_PUSH_UPLOAD_MAX_SIZE: {}", value);
            if value == UPLOAD_MAX_SIZE_UNLIMITED {
                return None;
            }

            return Some(value.parse::<u64>().expect("upload max size must be a number"))
        }

        let value = self.max_size.clone();
        if let Some(value) = value {
            debug!("using upload.max_size configuration: {}", value);
            return Some(value)
        }

        None
    }

    pub fn directory(&self) -> String {
        if let Some(value) = env("MIKU_PUSH_UPLOAD_DIRECTORY") {
            debug!("using env variable MIKU_PUSH_UPLOAD_DIRECTORY: {}", value);
            return value
        }

        let value = self.directory.clone();
        if let Some(value) = value {
            debug!("using upload.directory configuration: {}", value);
            return value
        }

        "data".to_string()
    }
}

impl Default for Upload {
    fn default() -> Self {
        Self {
            max_size: None,
            directory: None,
        }
    }
}