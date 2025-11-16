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

use std::path::{Path, PathBuf};
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::config::Settings;
use crate::errors::FileUploadError;

#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = crate::schema::file_uploads)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FileUpload {
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub uploaded_at: NaiveDateTime,
    pub chunked: bool
}

impl FileUpload {
    /// Create and retrieve the directory for the file upload
    pub fn directory(&self, settings: &Settings) -> Result<PathBuf, std::io::Error> {
        let destination_directory = settings.upload.directory();
        let destination_directory = Path::new(destination_directory.as_str())
            .join(self.id.to_string());

        if let Err(err) = std::fs::create_dir_all(&destination_directory) {
            return Err(err)
        }

        Ok(destination_directory)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileStatus {
    WaitingForUpload,
    Uploaded
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub uploaded_at: NaiveDateTime,
    pub status: FileStatus
}

impl FileInfo {
    pub fn from_file_upload(file_upload: &FileUpload, path: PathBuf) -> Self {
        Self {
            id: file_upload.id,
            name: file_upload.name.clone(),
            mime_type: file_upload.mime_type.clone(),
            size: file_upload.size,
            uploaded_at: file_upload.uploaded_at,
            status: match path.exists() {
                true => FileStatus::Uploaded,
                false => FileStatus::WaitingForUpload,
            }
        }
    }
}
