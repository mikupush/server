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
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FileUpload {
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub uploaded_at: NaiveDateTime,
    pub chunked: bool
}

impl FileUpload {
    pub fn new(id: Uuid, name: String, mime_type: String, size: i64, uploaded_at: NaiveDateTime) -> Self {
        Self { id, name, mime_type, size, uploaded_at, chunked: false }
    }

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

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, PartialEq, Eq, Hash)]
#[diesel(table_name = crate::schema::file_uploads)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FileUploadModel {
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub uploaded_at: NaiveDateTime,
    pub chunked: bool
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
                chunked: false
            }
        }
    }
}
