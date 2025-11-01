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

use std::fmt::Display;
use crate::errors::Error;
use uuid::Uuid;

#[derive(Debug)]
pub enum FileDeleteError {
    NotExists { id: Uuid },
    IO { message: String },
    DB { message: String }
}

impl Error for FileDeleteError {
    fn code(&self) -> String {
        match self {
            Self::NotExists { .. } => file_delete_codes::NOT_EXISTS_CODE.to_string(),
            Self::DB { .. } => file_delete_codes::DB_CODE.to_string(),
            Self::IO { .. } => file_delete_codes::IO_CODE.to_string(),
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

impl Display for FileDeleteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} error: {}", self.code(), self.message())
    }
}

impl From<std::io::Error> for FileDeleteError {
    fn from(value: std::io::Error) -> Self {
        FileDeleteError::IO { message: value.to_string() }
    }
}

impl From<diesel::result::Error> for FileDeleteError {
    fn from(value: diesel::result::Error) -> Self {
        Self::DB { message: value.to_string() }
    }
}

impl From<r2d2::Error> for FileDeleteError {
    fn from(value: r2d2::Error) -> Self {
        Self::DB { message: value.to_string() }
    }
}

pub mod file_delete_codes {
    pub const NOT_EXISTS_CODE: &str = "NotExists";
    pub const DB_CODE: &str = "DB";
    pub const IO_CODE: &str = "IO";
}