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
use crate::database::DbPool;
use crate::errors::FileReadError;
use crate::model::FileUpload;
use crate::schema::file_uploads;
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Stream;
use tokio::fs::File;
use std::io::Read;
use actix_web::body::SizedStream;
use tokio_util::io::ReaderStream;
use tracing::{debug, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FileReader {
    pool: DbPool,
    settings: Settings,
}

impl FileReader {
    pub fn new(pool: DbPool, settings: Settings) -> Self {
        Self { pool, settings }
    }

    pub async fn read(&self, id: Uuid) -> Result<FileRead, FileReadError> {
        let mut connection = self.pool.get()?;
        let file_upload: Option<FileUpload> = file_uploads::table
            .find(id)
            .first(&mut connection)
            .optional()?;

        let Some(file_upload) = file_upload else {
            return Err(FileReadError::NotExists { id })
        };

        let directory = file_upload.directory(&self.settings)?;
        let path = Path::new(&directory).join(file_upload.name.clone()).to_string_lossy().to_string();
        let file = File::open(path.clone()).await?;

        Ok(FileRead::from(file, &file_upload))
    }
}

pub struct FileRead {
    pub name: String,
    pub mime_type: String,
    pub size: u64,
    pub stream: ReaderStream<File>,
}

impl FileRead {
    fn from(file: File, file_upload: &FileUpload) -> Self {
        Self {
            name: file_upload.name.clone(),
            mime_type: file_upload.mime_type.clone(),
            size: file_upload.size as u64,
            stream: ReaderStream::new(file)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::config::Settings;
    use crate::database::DbPool;
    use crate::services::FileReader;

    impl FileReader {
        pub fn test(pool: DbPool) -> Self {
            Self { pool, settings: Settings::default() }
        }
    }
}
