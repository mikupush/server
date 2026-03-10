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
use crate::errors::FileReadError;
use crate::model::{FilePart, FileUpload};
use crate::repository::{FileUploadRepository, PostgresFileUploadRepository};
use crate::services::object_storage_reader::{FileSystemObjectStorageReader, ObjectStorageReader};
use bytes::Bytes;
use rusqlite::fallible_iterator::FallibleIterator;
use std::io;
use std::path::Path;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use uuid::Uuid;
use crate::cache::MokaCache;

#[derive(Debug, Clone)]
pub struct FileReader<FR, OBR>
where
    FR: FileUploadRepository + Clone + Send + 'static,
    OBR: ObjectStorageReader + Clone + Send + 'static,
{
    repository: FR,
    reader: OBR,
    settings: Settings,
}

impl<FR, OBR> FileReader<FR, OBR>
where
    FR: FileUploadRepository + Clone + Send + 'static,
    OBR: ObjectStorageReader + Clone + Send,
{
    pub fn new(repository: FR, reader: OBR, settings: &Settings) -> Self {
        Self {
            repository,
            reader,
            settings: settings.clone()
        }
    }

    pub async fn read(&self, id: Uuid) -> Result<FileStreamWrapper, FileReadError> {
        let repository = self.repository.clone();
        let file_upload_result = tokio::task::spawn_blocking(move || repository.find_by_id(&id)).await
            .map_err(|err| FileReadError::IO { message: err.to_string() })?;
        let file_upload = match file_upload_result? {
            Some(file_upload) => file_upload,
            None => return Err(FileReadError::NotExists { id })
        };

        if file_upload.chunked {
            let reader = ChunkedFileReader::new(
                self.reader.clone(),
                file_upload.clone(),
                self.settings.clone()
            );

            reader.read().await
        } else {
            let reader = SingleFileReader {
                details: file_upload,
                settings: self.settings.clone(),
                reader: self.reader.clone()
            };

            reader.read().await
        }
    }
}

impl FileReader<PostgresFileUploadRepository<MokaCache>, FileSystemObjectStorageReader> {
    pub fn get_with_settings(settings: &Settings) -> Self {
        Self::new(
            PostgresFileUploadRepository::get_with_settings(&settings),
            FileSystemObjectStorageReader::new(),
            settings
        )
    }
}

pub struct FileStreamWrapper {
    pub details: FileUpload,
    pub stream: Box<dyn Stream<Item = io::Result<Bytes>> + Send + Unpin + 'static>,
}

#[derive(Clone)]
pub struct SingleFileReader<OBR>
where
    OBR: ObjectStorageReader + Clone + Send + 'static
{
    pub details: FileUpload,
    pub settings: Settings,
    pub reader: OBR
}

impl<OBR> SingleFileReader<OBR>
where
    OBR: ObjectStorageReader + Clone + Send + 'static
{
    pub async fn read(&self) -> Result<FileStreamWrapper, FileReadError> {
        let directory = self.details.content_directory(&self.settings)?;
        let path = Path::new(&directory)
            .join(self.details.name.clone())
            .to_string_lossy()
            .to_string();
        let stream = self.reader.read(path).await?;

        Ok(FileStreamWrapper {
            details: self.details.clone(),
            stream: Box::new(stream)
        })
    }
}

#[derive(Debug, Clone)]
pub struct ChunkedFileReader<OBR>
where
    OBR: ObjectStorageReader + Clone + Send + 'static
{
    reader: OBR,
    last_index: i32,
    details: FileUpload,
    settings: Settings,
}

impl<OBR> ChunkedFileReader<OBR>
where
    OBR: ObjectStorageReader + Clone + Send + 'static
{
    pub fn new(reader: OBR, details: FileUpload, settings: Settings) -> Self {
        Self {
            reader,
            details,
            settings,
            last_index: -1,
        }
    }

    pub async fn read(&self) -> Result<FileStreamWrapper, FileReadError> {
        let (sender, receiver) = mpsc::channel::<io::Result<Bytes>>(1);

        let mut reader = self.clone();
        tokio::spawn(async move {
            let _ = reader.send_bytes(sender).await;
        });

        Ok(FileStreamWrapper {
            details: self.details.clone(),
            stream: Box::new(ReceiverStream::new(receiver))
        })
    }

    async fn send_bytes(&mut self, sender: mpsc::Sender<io::Result<Bytes>>) -> io::Result<()> {
        let directory = self.details.content_directory(&self.settings)?;
        let parts = std::fs::read_dir(&directory)?.count();
        let directory = directory.clone();

        for part in 0..parts {
            let location = directory.join(FilePart::name(part))
                .to_string_lossy()
                .to_string();

            let bytes = self.reader.read_all(location).await;
            let _ = sender.send(bytes).await;
        }

        Ok(())
    }
}
