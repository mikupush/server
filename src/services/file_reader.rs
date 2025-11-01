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
use std::fs::File;
use std::io::Read;
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

    pub fn read(&self, id: Uuid) -> Result<FileReadStream, FileReadError> {
        let mut connection = self.pool.get()?;
        let file_upload: Option<FileUpload> = file_uploads::table
            .find(id)
            .first(&mut connection)
            .optional()?;

        let Some(file_upload) = file_upload else {
            return Err(FileReadError::NotExists { id })
        };

        let directory = self.settings.upload.directory();
        let path = Path::new(&directory).join(file_upload.name.clone()).to_string_lossy().to_string();
        let file = File::open(path.clone())?;

        Ok(FileReadStream::from(path.clone(), file, &file_upload))
    }
}

pub struct FileReadStream {
    id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: u64,
    path: String,
    file: File,
}

impl FileReadStream {
    fn from(path: String, file: File, file_upload: &FileUpload) -> Self {
        Self {
            id: file_upload.id.clone(),
            name: file_upload.name.clone(),
            mime_type: file_upload.mime_type.clone(),
            size: file_upload.size as u64,
            path,
            file
        }
    }
}

impl Stream for FileReadStream {
    type Item = Result<actix_web::web::Bytes, std::io::Error>;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let id = self.id.clone();
        let path = self.path.clone();
        let file = &mut self.get_mut().file;
        let mut buffer = vec![0u8; 1024];

        match file.read(&mut buffer) {
            Ok(0) => {
                debug!("stream reached end for file with id {} ({})", id, path);
                Poll::Ready(None)
            },
            Ok(bytes_read) => {
                debug!("read {} bytes during file with id {} ({}) stream", bytes_read, id, path);
                buffer.truncate(bytes_read);
                let bytes = actix_web::web::Bytes::from(buffer);
                Poll::Ready(Some(Ok(bytes)))
            },
            Err(err) => {
                warn!("error during file read stream for file with id {} ({}): {}", id, path, err);
                Poll::Ready(None)
            },
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