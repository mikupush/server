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

use crate::errors::Error;
use std::fmt::Display;
use uuid::Uuid;

#[derive(Debug)]
pub enum FileUploadError {
    Exists,
    NotExists { id: Uuid },
    MaxFileSizeExceeded,
    NotCompleted,
    StreamRead { message: String },
    IO { message: String },
    DB { message: String }
}

impl Display for FileUploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} error: {}", self.code(), self.message())
    }
}

impl Error for FileUploadError {
    fn code(&self) -> String {
        match self {
            Self::Exists => file_upload_codes::EXISTS_CODE.to_string(),
            Self::NotExists { .. } => file_upload_codes::NOT_EXISTS_CODE.to_string(),
            Self::MaxFileSizeExceeded => file_upload_codes::MAX_FILE_SIZE_EXCEEDED_CODE.to_string(),
            Self::StreamRead { .. } => file_upload_codes::STREAM_READ_CODE.to_string(),
            Self::DB { .. } => file_upload_codes::DB_CODE.to_string(),
            Self::IO { .. } => file_upload_codes::IO_CODE.to_string(),
            Self::NotCompleted => file_upload_codes::NOT_COMPLETED_CODE.to_string(),
        }
    }

    fn message(&self) -> String {
        match self {
            Self::Exists => "File is already registered".to_string(),
            Self::NotExists { id: uuid } => format!("File with uuid {} is not registered", uuid),
            Self::MaxFileSizeExceeded => "Max file size exceeded".to_string(),
            Self::StreamRead { message } => format!("Error reading uploaded file stream: {}", message),
            Self::DB { message } => message.clone(),
            Self::IO { message } => message.clone(),
            Self::NotCompleted => "File upload is not completed".to_string(),
        }
    }
}

impl From<actix_web::error::PayloadError> for FileUploadError {
    fn from(value: actix_web::error::PayloadError) -> Self {
        Self::StreamRead { message: value.to_string() }
    }
}

impl From<std::io::Error> for FileUploadError {
    fn from(value: std::io::Error) -> Self {
        Self::IO { message: value.to_string() }
    }
}

impl From<diesel::result::Error> for FileUploadError {
    fn from(value: diesel::result::Error) -> Self {
        Self::DB { message: value.to_string() }
    }
}

impl From<r2d2::Error> for FileUploadError {
    fn from(value: r2d2::Error) -> Self {
        Self::DB { message: value.to_string() }
    }
}

pub mod file_upload_codes {
    pub const EXISTS_CODE: &str = "Exists";
    pub const NOT_EXISTS_CODE: &str = "NotExists";
    pub const MAX_FILE_SIZE_EXCEEDED_CODE: &str = "MaxFileSizeExceeded";
    pub const NOT_COMPLETED_CODE: &str = "NotCompleted";
    pub const STREAM_READ_CODE: &str = "StreamRead";
    pub const DB_CODE: &str = "DB";
    pub const IO_CODE: &str = "IO";
}