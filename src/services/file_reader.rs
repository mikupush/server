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
use crate::model::{FileUpload, Part};
use crate::repository::{FileUploadRepository, ManifestRepository, PostgresFileUploadRepository, SQLiteManifestRepository};
use crate::services::object_storage_reader::{FileSystemObjectStorageReader, ObjectStorageReader};
use bytes::Bytes;
use futures::future::BoxFuture;
use futures::FutureExt;
use futures::StreamExt;
use rusqlite::fallible_iterator::FallibleIterator;
use std::io;
use std::path::Path;
use tokio::fs::File;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FileReader<FR, OBR, MR>
where
    FR: FileUploadRepository + Clone + 'static,
    OBR: ObjectStorageReader + Clone + Send + 'static,
    MR: ManifestRepository + Clone + Send + 'static,
{
    repository: FR,
    reader: OBR,
    manifest_repository: MR,
    settings: Settings,
}

impl<FR, OBR, MR> FileReader<FR, OBR, MR>
where
    FR: FileUploadRepository + Clone,
    OBR: ObjectStorageReader + Clone + Send,
    MR: ManifestRepository + Clone + Send,
{
    pub fn new(repository: FR, reader: OBR, manifest_repository: MR, settings: Settings) -> Self {
        Self { repository, reader, manifest_repository, settings }
    }

    pub async fn read(&self, id: Uuid) -> Result<FileStreamWrapper, FileReadError> {
        let file_upload = match self.repository.find_by_id(id)? {
            Some(file_upload) => file_upload,
            None => return Err(FileReadError::NotExists { id })
        };

        if file_upload.chunked {
            let reader = ChunkedFileReader::new(
                self.manifest_repository.clone(),
                self.reader.clone(),
                file_upload.clone(),
                self.settings.clone()
            );

            reader.read().await
        } else {
            let reader = SingleFileReader {
                details: file_upload,
                settings: self.settings.clone()
            };

            reader.read().await
        }
    }
}

impl FileReader<PostgresFileUploadRepository, FileSystemObjectStorageReader, SQLiteManifestRepository> {
    pub fn get_with_settings(settings: Settings) -> Self {
        Self::new(
            PostgresFileUploadRepository::get_with_settings(settings.clone()),
            FileSystemObjectStorageReader::new(),
            SQLiteManifestRepository::new(settings.clone()),
            settings
        )
    }
}

type StreamBuilderResult = io::Result<Box<dyn Stream<Item = io::Result<Bytes>>>>;
type StreamBuilder = Box<dyn Fn() -> BoxFuture<'static, StreamBuilderResult>>;

pub struct FileStreamWrapper {
    pub details: FileUpload,
    pub stream: Box<dyn Stream<Item = io::Result<Bytes>> + Send + Unpin + 'static>,
}

#[derive(Clone)]
pub struct SingleFileReader {
    pub details: FileUpload,
    pub settings: Settings,
}

impl SingleFileReader {
    pub async fn read(&self) -> Result<FileStreamWrapper, FileReadError> {
        let directory = self.details.directory(&self.settings)?;
        let path = Path::new(&directory)
            .join(self.details.name.clone())
            .to_string_lossy()
            .to_string();
        let file = File::open(path.clone()).await?;

        Ok(FileStreamWrapper {
            details: self.details.clone(),
            stream: Box::new(ReaderStream::new(file))
        })
    }
}

#[derive(Debug, Clone)]
pub struct ChunkedFileReader<MR, OBR>
where
    MR: ManifestRepository + Clone + Send + 'static,
    OBR: ObjectStorageReader + Clone + Send + 'static
{
    repository: MR,
    reader: OBR,
    last_index: i32,
    details: FileUpload,
    settings: Settings
}

impl<MR, OBR> ChunkedFileReader<MR, OBR>
where
    MR: ManifestRepository + Clone + Send + 'static,
    OBR: ObjectStorageReader + Clone + Send + 'static
{
    pub fn new(repository: MR, reader: OBR, details: FileUpload, settings: Settings) -> Self {
        Self {
            repository,
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

    fn next_part(&mut self) -> io::Result<Option<Part>> {
        let parts = self.repository.take_parts(self.details.id.clone(), 1, self.last_index);
        if let Err(err) = parts {
            return Err(io::Error::new(io::ErrorKind::Other, err.to_string()));
        }

        let part = parts
            .unwrap()
            .first()
            .map(|part| part.clone());

        self.last_index += 1;
        Ok(part)
    }

    async fn send_bytes(&mut self, sender: mpsc::Sender<io::Result<Bytes>>) -> io::Result<()> {
        let directory = self.details.directory(&self.settings)?;
        let directory = directory.clone();

        loop {
            let part = self.next_part();

            if let Err(err) = part {
                let _ = sender.send(Err(err)).await;
                continue;
            }

            let Some(part) = part.unwrap() else {
                break;
            };

            let location = directory.join(part.file_name())
                .to_string_lossy()
                .to_string();

            let bytes = self.reader.read_all(location).await;
            let _ = sender.send(bytes).await;
        }

        Ok(())
    }
}
