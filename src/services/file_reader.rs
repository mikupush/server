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
