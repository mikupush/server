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
use crate::errors::Error;
use crate::repository::{FileUploadRepository, FileUploadRepositoryError, PostgresFileUploadRepository};
use crate::services::file_writer::{FileSystemFileWriter, FileWriteError, FileWriter};
use crate::services::FileSizeLimiter;
use actix_web::web::Payload;
use futures::StreamExt;
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::Write;
use tokio::io::AsyncRead;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FileUploader<FR, FW>
where
    FR: FileUploadRepository + Clone,
    FW: FileWriter + Clone,
{
    repository: FR,
    writer: FW,
    settings: Settings,
    limiter: FileSizeLimiter
}

impl<FR, FW> FileUploader<FR, FW>
where
    FR: FileUploadRepository + Clone,
    FW: FileWriter + Clone,
{
    pub fn new(
        repository: FR,
        writer: FW,
        settings: Settings,
        limiter: FileSizeLimiter
    ) -> Self {
        Self { repository, settings, limiter, writer }
    }

    pub async fn upload_file(&self, id: Uuid, reader: impl AsyncRead + Unpin) -> Result<(), FileUploadError> {
        let file_upload = match self.repository.find_by_id(id)? {
            Some(file_upload) => file_upload,
            None => {
                debug!("file upload {} does not exist", id);
                return Err(FileUploadError::NotExists { id })
            }
        };

        let destination_path = file_upload.directory(&self.settings)?;
        let destination_path = destination_path.join(file_upload.name.clone());
        let destination_path = destination_path.to_string_lossy().to_string();

        let bytes_written = if self.settings.upload.is_limited() {
            self.writer.write(reader, destination_path, self.settings.upload.max_size()).await?
        } else {
            self.writer.write(reader, destination_path, None).await?
        };

        if self.limiter.check_file_size(bytes_written) == false {
            debug!("file size limit exceeded");
            return Err(FileUploadError::MaxFileSizeExceeded);
        }

        Ok(())
    }
}

impl FileUploader<PostgresFileUploadRepository, FileSystemFileWriter> {
    pub fn get_with_settings(settings: Settings) -> Self {
        Self::new(
            PostgresFileUploadRepository::get_with_settings(settings.clone()),
            FileSystemFileWriter::new(),
            settings.clone(),
            FileSizeLimiter::new(settings.clone())
        )
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum FileUploadError {
    Exists,
    NotExists { id: Uuid },
    MaxFileSizeExceeded,
    NotCompleted,
    StreamRead { message: String },
    IO { message: String },
    DB { message: String }
}

impl Display for FileUploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} error: {}", self.code(), self.message())
    }
}

impl Error for FileUploadError {
    fn code(&self) -> String {
        match self {
            Self::Exists => file_upload_codes::EXISTS_CODE.to_string(),
            Self::NotExists { .. } => file_upload_codes::NOT_EXISTS_CODE.to_string(),
            Self::MaxFileSizeExceeded => file_upload_codes::MAX_FILE_SIZE_EXCEEDED_CODE.to_string(),
            Self::StreamRead { .. } => file_upload_codes::STREAM_READ_CODE.to_string(),
            Self::DB { .. } => file_upload_codes::DB_CODE.to_string(),
            Self::IO { .. } => file_upload_codes::IO_CODE.to_string(),
            Self::NotCompleted => file_upload_codes::NOT_COMPLETED_CODE.to_string(),
        }
    }

    fn message(&self) -> String {
        match self {
            Self::Exists => "File is already registered".to_string(),
            Self::NotExists { id: uuid } => format!("File with uuid {} is not registered", uuid),
            Self::MaxFileSizeExceeded => "Max file size exceeded".to_string(),
            Self::StreamRead { message } => format!("Error reading uploaded file stream: {}", message),
            Self::DB { message } => message.clone(),
            Self::IO { message } => message.clone(),
            Self::NotCompleted => "File upload is not completed".to_string(),
        }
    }
}

impl From<actix_web::error::PayloadError> for FileUploadError {
    fn from(value: actix_web::error::PayloadError) -> Self {
        Self::StreamRead { message: value.to_string() }
    }
}

impl From<std::io::Error> for FileUploadError {
    fn from(value: std::io::Error) -> Self {
        Self::IO { message: value.to_string() }
    }
}

impl From<diesel::result::Error> for FileUploadError {
    fn from(value: diesel::result::Error) -> Self {
        Self::DB { message: value.to_string() }
    }
}

impl From<r2d2::Error> for FileUploadError {
    fn from(value: r2d2::Error) -> Self {
        Self::DB { message: value.to_string() }
    }
}

impl From<FileUploadRepositoryError> for FileUploadError {
    fn from(value: FileUploadRepositoryError) -> Self {
        match value {
            FileUploadRepositoryError::Db(err) => err.into(),
            FileUploadRepositoryError::Pool(err) => err.into(),
        }
    }
}

impl From<FileWriteError> for FileUploadError {
    fn from(value: FileWriteError) -> Self {
        match value {
            FileWriteError::Io(err) => FileUploadError::IO { message: err.to_string() },
        }
    }
}

pub mod file_upload_codes {
    pub const EXISTS_CODE: &str = "Exists";
    pub const NOT_EXISTS_CODE: &str = "NotExists";
    pub const MAX_FILE_SIZE_EXCEEDED_CODE: &str = "MaxFileSizeExceeded";
    pub const NOT_COMPLETED_CODE: &str = "NotCompleted";
    pub const STREAM_READ_CODE: &str = "StreamRead";
    pub const DB_CODE: &str = "DB";
    pub const IO_CODE: &str = "IO";
}

#[cfg(test)]
mod tests {
    use crate::config::Settings;
    use crate::domain::FileUpload;
    use crate::repository::InMemoryFileUploadRepository;
    use crate::services::file_writer::FakeFileWriter;
    use crate::services::{FileSizeLimiter, FileUploadError, FileUploader};
    use bytes::Bytes;
    use std::collections::HashMap;
    use tokio_util::io::StreamReader;
    use uuid::Uuid;

    impl FileUploader<InMemoryFileUploadRepository, FakeFileWriter> {
        pub fn create() -> Self {
            Self {
                repository: Self::create_repository(),
                writer: FakeFileWriter,
                settings: Settings::default(),
                limiter: FileSizeLimiter::create()
            }
        }

        pub fn create_with_limit() -> Self {
            Self {
                repository: Self::create_repository(),
                writer: FakeFileWriter,
                settings: Settings::default(),
                limiter: FileSizeLimiter::create_limited()
            }
        }

        fn create_repository() -> InMemoryFileUploadRepository {
            let items = HashMap::from([
                (
                    Uuid::parse_str("5769aa43-2380-49be-aafb-e9dd4bd4564f").unwrap(),
                    FileUpload::create("5769aa43-2380-49be-aafb-e9dd4bd4564f")
                ),
            ]);

            InMemoryFileUploadRepository::new(items)
        }
    }

    #[actix_web::test]
    async fn test_upload_file() {
        let uploader = FileUploader::create();
        let id = Uuid::parse_str("5769aa43-2380-49be-aafb-e9dd4bd4564f").unwrap();
        let stream = tokio_stream::iter(vec![
            tokio::io::Result::Ok(Bytes::from("Hello")),
            tokio::io::Result::Ok(Bytes::from("World")),
        ]);

        let reader = StreamReader::new(stream);
        let result = uploader.upload_file(id, reader).await;

        assert_eq!(true, result.is_ok());
    }

    #[actix_web::test]
    async fn test_upload_file_not_exists() {
        let uploader = FileUploader::create();
        let id = Uuid::parse_str("f5dcca7d-e8ba-4e87-b7b1-e3cde6bc857d").unwrap();
        let stream = tokio_stream::iter(vec![
            tokio::io::Result::Ok(Bytes::from("Hello")),
            tokio::io::Result::Ok(Bytes::from("World")),
        ]);

        let reader = StreamReader::new(stream);
        let result = uploader.upload_file(id, reader).await;

        assert_eq!(true, result.is_err());
        assert_eq!(FileUploadError::NotExists { id }, result.unwrap_err());
    }

    #[actix_web::test]
    async fn test_upload_file_max_file_size_exceeded() {
        let uploader = FileUploader::create_with_limit();
        let id = Uuid::parse_str("5769aa43-2380-49be-aafb-e9dd4bd4564f").unwrap();
        let content = vec![1u8; 200];
        let stream = tokio_stream::iter(vec![
            tokio::io::Result::Ok(Bytes::from(content)),
        ]);

        let reader = StreamReader::new(stream);
        let result = uploader.upload_file(id, reader).await;

        assert_eq!(true, result.is_err());
        assert_eq!(FileUploadError::MaxFileSizeExceeded, result.unwrap_err());
    }
}
