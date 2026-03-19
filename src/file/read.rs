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

use crate::cache::MokaCache;
use crate::config::Settings;
use crate::file::error::FileReadError;
use crate::file::upload::FileUpload;
use crate::file::FilePart;
use crate::file::{FileUploadRepository, PostgresFileUploadRepository};
use crate::storage::{FileSystemObjectStorageReader, ObjectStorageReader, ObjectStorageReaderFactory};
use bytes::Bytes;
use futures::future::Either;
use futures::stream::{self, BoxStream, StreamExt};
use futures::TryStreamExt;
use rusqlite::fallible_iterator::FallibleIterator;
use std::io;
use std::io::Read;
use std::path::Path;
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tracing::error;
use uuid::Uuid;

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
        let file_upload = self.find_upload_by_id(&id).await?;

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

    pub async fn read_range(&self, id: Uuid, start: u64, end: u64) -> Result<FileStreamWrapper, FileReadError> {
        let file_upload = self.find_upload_by_id(&id).await?;
        let size = file_upload.size as u64;

        if file_upload.chunked {
            return Err(FileReadError::RangeNotAllowed(id, "file is chunked".to_string()));
        }

        if start > end {
            return Err(FileReadError::RangeNotAllowed(id, "start is greater than end".to_string()));
        }

        if start >= size || end >= size {
            return Err(FileReadError::RangeNotAllowed(id, "range indices are out of file bounds".to_string()));
        }

        let reader = SingleFileReader {
            details: file_upload,
            settings: self.settings.clone(),
            reader: self.reader.clone()
        };

        Ok(reader.read_range(start, end).await?)
    }

    async fn find_upload_by_id(&self, id: &Uuid) -> Result<FileUpload, FileReadError> {
        let repository = self.repository.clone();
        let id = id.clone();

        let find_task = tokio::task::spawn_blocking(move || {
            repository.find_by_id(&id)
        });

        let file_upload_result = find_task.await
            .map_err(|err| FileReadError::IO { message: err.to_string() })?;

        match file_upload_result? {
            Some(file_upload) => Ok(file_upload),
            None => Err(FileReadError::NotExists { id })
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

const CHUNK_READ_SIZE: usize = 64 * 1024;

pub struct FileStreamWrapper {
    pub details: FileUpload,
    pub stream: ReceiverStream<io::Result<Bytes>>,
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
        let details = self.details.clone();
        let reader = self.reader.clone();
        let directory = self.details.content_directory(&self.settings)?;
        let path = Path::new(&directory)
            .join(details.name.clone())
            .to_string_lossy()
            .to_string();

        let (sender, receiver) = mpsc::channel::<io::Result<Bytes>>(10);
        let mut stream = tokio::task::spawn_blocking(move || reader.read(&path)).await
            .map_err(|err| FileReadError::IO { message: err.to_string() })??;

        tokio::task::spawn_blocking(move || {
            send_reader_bytes(&mut stream, &sender);
        });

        Ok(FileStreamWrapper {
            details: self.details.clone(),
            stream: ReceiverStream::new(receiver),
        })
    }

    pub async fn read_range(&self, start: u64, end: u64) -> Result<FileStreamWrapper, FileReadError> {
        let details = self.details.clone();
        let reader = self.reader.clone();
        let directory = self.details.content_directory(&self.settings)?;
        let path = Path::new(&directory)
            .join(details.name.clone())
            .to_string_lossy()
            .to_string();

        let (sender, receiver) = mpsc::channel::<io::Result<Bytes>>(10);
        let task = tokio::task::spawn_blocking(move || {
            reader.read_range(&path, start, end)
        });

        let mut stream = task.await.map_err(|err| FileReadError::IO { message: err.to_string() })??;

        tokio::task::spawn_blocking(move || {
            send_reader_bytes(&mut stream, &sender);
        });

        Ok(FileStreamWrapper {
            details: self.details.clone(),
            stream: ReceiverStream::new(receiver),
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
        let (sender, receiver) = mpsc::channel::<io::Result<Bytes>>(10);
        let this = self.clone();

        tokio::task::spawn_blocking(move || {
            if let Err(err) = this.send_bytes(sender) {
                error!("error sending file bytes to stream: {}", err);
            }
        });

        Ok(FileStreamWrapper {
            details: self.details.clone(),
            stream: ReceiverStream::new(receiver),
        })
    }

    fn send_bytes(self, sender: Sender<io::Result<Bytes>>) -> Result<(), FileReadError> {
        let directory = self.details.content_directory(&self.settings)?;
        let parts = std::fs::read_dir(&directory)?.count();

        for part in 0..parts {
            let location = directory.join(FilePart::name(part))
                .to_string_lossy()
                .to_string();

            let mut stream = self.reader.read(&location)?;
            send_reader_bytes(&mut stream, &sender);
        }

        Ok(())
    }
}

fn send_reader_bytes(reader: &mut impl Read, sender: &Sender<io::Result<Bytes>>) {
    let mut buffer = [0u8; CHUNK_READ_SIZE];

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                let bytes = Bytes::copy_from_slice(&buffer[..n]);
                if sender.blocking_send(Ok(bytes)).is_err() { break; }
            }
            Err(e) => {
                let _ = sender.blocking_send(Err(e));
                break;
            }
        }
    }
}
