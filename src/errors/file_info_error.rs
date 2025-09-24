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

use std::fmt::Display;
use crate::errors::Error;
use uuid::Uuid;

#[derive(Debug)]
pub enum FileInfoError {
    NotExists { id: Uuid },
    IO { message: String },
    DB { message: String }
}

impl Error for FileInfoError {
    fn code(&self) -> String {
        match self {
            Self::NotExists { .. } => file_info_codes::NOT_EXISTS_CODE.to_string(),
            Self::DB { .. } => file_info_codes::DB_CODE.to_string(),
            Self::IO { .. } => file_info_codes::IO_CODE.to_string(),
        }
    }

    fn message(&self) -> String {
        match self {
            Self::NotExists { id: uuid } => format!("File with uuid {} is not registered", uuid),
            Self::DB { message } => message.clone(),
            Self::IO { message } => message.clone(),
        }
    }
}

impl Display for FileInfoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} error: {}", self.code(), self.message())
    }
}

impl From<std::io::Error> for FileInfoError {
    fn from(value: std::io::Error) -> Self {
        FileInfoError::IO { message: value.to_string() }
    }
}

impl From<diesel::result::Error> for FileInfoError {
    fn from(value: diesel::result::Error) -> Self {
        Self::DB { message: value.to_string() }
    }
}

impl From<r2d2::Error> for FileInfoError {
    fn from(value: r2d2::Error) -> Self {
        Self::DB { message: value.to_string() }
    }
}

pub mod file_info_codes {
    pub const NOT_EXISTS_CODE: &str = "NotExists";
    pub const DB_CODE: &str = "DB";
    pub const IO_CODE: &str = "IO";
}
