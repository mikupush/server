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

use std::fmt::{Display, Formatter};
use std::path::Path;
use tokio_util::io::StreamReader;
use uuid::Uuid;
use crate::cache::MokaCache;
use crate::config::Settings;
use crate::file::{FileUploadRepository, FileUploadRepositoryError, PostgresFileUploadRepository};
use crate::storage::{FileSystemObjectStorageAppender, FileSystemObjectStorageReader, ObjectStorageAppendError, ObjectStorageAppender, ObjectStorageReader};

pub struct FilePart;

impl FilePart {
    pub fn name(index: usize) -> String {
        format!("{}.part", index)
    }
}

#[derive(Clone)]
pub struct FileAssembler<FR, OBR, OBA>
where
    FR: FileUploadRepository + Clone,
    OBR: ObjectStorageReader + Clone,
    OBA: ObjectStorageAppender + Clone,
{
    settings: Settings,
    repository: FR,
    reader: OBR,
    appender: OBA,
}

impl<FR, OBR, OBA> FileAssembler<FR, OBR, OBA>
where
    FR: FileUploadRepository + Clone,
    OBR: ObjectStorageReader + Clone,
    OBA: ObjectStorageAppender + Clone,
{
    pub fn new(settings: &Settings, repository: FR, reader: OBR, appender: OBA) -> Self {
        Self { settings: settings.clone(), repository, reader, appender }
    }

    pub fn assemble(&self, id: &Uuid) -> Result<(), FileAssembleError> {
        let Some(file_upload) = self.repository.find_by_id(id)? else {
            return Err(FileAssembleError::NotFound(id.clone()));
        };

        let mut file_upload = file_upload;
        if !file_upload.chunked {
            return Err(FileAssembleError::NotChunked(id.clone()));
        }

        let directory = file_upload.content_directory(&self.settings)?;
        let assembled_name = Path::new(&directory).join(&file_upload.name)
            .to_string_lossy()
            .to_string();

        let parts = std::fs::read_dir(&directory)?.count();

        for part in 0..parts {
            let name = FilePart::name(part);
            let location = Path::new(&directory).join(name)
                .to_string_lossy()
                .to_string();

            let read_stream = self.reader.read(&location)?;
            self.appender.append(read_stream, assembled_name.clone())?;
        }

        for part in 0..parts {
            let name = FilePart::name(part);
            let location = Path::new(&directory).join(name);
            std::fs::remove_file(location)?;
        }

        // remove chunked flag for this upload
        file_upload.chunked = false;
        self.repository.save(&file_upload)?;

        Ok(())
    }
}

impl FileAssembler<
    PostgresFileUploadRepository<MokaCache>,
    FileSystemObjectStorageReader,
    FileSystemObjectStorageAppender
> {
    pub fn get_with_settings(settings: &Settings) -> Self {
        Self {
            settings: settings.clone(),
            repository: PostgresFileUploadRepository::get_with_settings(&settings),
            reader: FileSystemObjectStorageReader::new(),
            appender: FileSystemObjectStorageAppender::new(),
        }
    }
}

pub enum FileAssembleError {
    IO(String),
    NotFound(Uuid),
    NotChunked(Uuid),
}

impl From<FileUploadRepositoryError> for FileAssembleError {
    fn from(value: FileUploadRepositoryError) -> Self {
        match value {
            FileUploadRepositoryError::Pool(err) => Self::IO(err.to_string()),
            FileUploadRepositoryError::Db(err) => Self::IO(err.to_string()),
        }
    }
}

impl From<std::io::Error> for FileAssembleError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value.to_string())
    }
}

impl From<ObjectStorageAppendError> for FileAssembleError {
    fn from(value: ObjectStorageAppendError) -> Self {
        match value {
            ObjectStorageAppendError::IO(err) => Self::IO(err.to_string()),
        }
    }
}

impl Display for FileAssembleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(err) => write!(f, "io error: {}", err),
            Self::NotFound(id) => write!(f, "file upload not found: {}", id),
            Self::NotChunked(id) => write!(f, "file upload is not chunked: {}", id),
        }
    }
}
