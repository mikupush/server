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

use crate::config::Settings;
use chrono::NaiveDateTime;
use diesel::{AsChangeset, Insertable, Queryable};
use std::path::{Path, PathBuf};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct FileUpload {
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub uploaded_at: NaiveDateTime,
    pub chunked: bool,
    pub expires_at: Option<NaiveDateTime>,
}

impl FileUpload {
    /// Create and retrieve the directory for the file upload
    fn directory(&self, settings: &Settings) -> Result<PathBuf, std::io::Error> {
        let destination_directory = settings.upload.directory.clone();
        let destination_directory = Path::new(destination_directory.as_str())
            .join(self.id.to_string());

        Ok(destination_directory)
    }

    /// Get the directory for the file content. This directory is used to store the file content
    /// which can be file parts or single file contents.
    pub fn content_directory(&self, settings: &Settings) -> Result<PathBuf, std::io::Error> {
        let destination_directory = self.directory(settings)?;
        let destination_directory = destination_directory.join("content");

        Ok(destination_directory)
    }

    /// Get the directory for the file checksum. This directory is used to store the file checksums.
    pub fn sum_directory(&self, settings: &Settings) -> Result<PathBuf, std::io::Error> {
        let destination_directory = self.directory(settings)?;
        let destination_directory = destination_directory.join("sum");

        Ok(destination_directory)
    }
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, PartialEq, Eq, Hash)]
#[diesel(table_name = crate::schema::file_uploads)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FileUploadModel {
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub uploaded_at: NaiveDateTime,
    pub chunked: bool,
    pub expires_at: Option<NaiveDateTime>,
}

impl From<FileUploadModel> for FileUpload {
    fn from(model: FileUploadModel) -> Self {
        Self {
            id: model.id,
            name: model.name,
            mime_type: model.mime_type,
            size: model.size,
            uploaded_at: model.uploaded_at,
            chunked: model.chunked,
            expires_at: model.expires_at,
        }
    }
}

impl From<FileUpload> for FileUploadModel {
    fn from(domain: FileUpload) -> Self {
        Self {
            id: domain.id,
            name: domain.name,
            mime_type: domain.mime_type,
            size: domain.size,
            uploaded_at: domain.uploaded_at,
            chunked: domain.chunked,
            expires_at: domain.expires_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::FileUpload;
    use uuid::Uuid;

    impl FileUpload {
        pub fn create(id: &str) -> Self {
            Self {
                id: Uuid::parse_str(id).unwrap(),
                name: "test.txt".to_string(),
                mime_type: "text/plain".to_string(),
                size: 10,
                uploaded_at: chrono::Utc::now().naive_utc(),
                chunked: false,
                expires_at: None,
            }
        }
    }
}
