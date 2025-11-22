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
use crate::repository::FileUploadRepositoryError;
use std::fmt::Display;
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

impl From<FileUploadRepositoryError> for FileInfoError {
    fn from(value: FileUploadRepositoryError) -> Self {
        match value {
            FileUploadRepositoryError::Db(err) => err.into(),
            FileUploadRepositoryError::Pool(err) => err.into(),
        }
    }
}

pub mod file_info_codes {
    pub const NOT_EXISTS_CODE: &str = "NotExists";
    pub const DB_CODE: &str = "DB";
    pub const IO_CODE: &str = "IO";
}
