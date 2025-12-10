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
use crate::model::{FileUpload, Manifest, Part};
use crate::repository::{FileUploadRepository, FileUploadRepositoryError, ManifestError, ManifestRepository, PostgresFileUploadRepository, SQLiteManifestRepository};
use crate::services::object_storage_remover::{FileSystemObjectStorageRemover, ObjectStorageRemoveError, ObjectStorageRemover};
use crate::services::object_storage_writer::{FileSystemObjectStorageWriter, ObjectStorageWriteError, ObjectStorageWriter};
use crate::services::FileSizeLimiter;
use std::fmt::Display;
use tokio::io::AsyncRead;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FileUploader<FR, MR, OSW, OSR>
where
    FR: FileUploadRepository + Clone + Send + 'static,
    MR: ManifestRepository + Clone + Send + 'static,
    OSW: ObjectStorageWriter + Clone + Send + 'static,
    OSR: ObjectStorageRemover + Clone + Send + 'static
{
    repository: FR,
    manifest_repository: MR,
    writer: OSW,
    remover: OSR,
    settings: Settings,
    limiter: FileSizeLimiter
}

impl<FR, MR, OSW, OSD> FileUploader<FR, MR, OSW, OSD>
where
    FR: FileUploadRepository + Clone + Send + 'static,
    MR: ManifestRepository + Clone + Send + 'static,
    OSW: ObjectStorageWriter + Clone + Send + 'static,
    OSD: ObjectStorageRemover + Clone + Send + 'static
{
    pub fn new(
        repository: FR,
        manifest_repository: MR,
        writer: OSW,
        remover: OSD,
        settings: Settings,
        limiter: FileSizeLimiter
    ) -> Self {
        Self { repository, manifest_repository, settings, limiter, writer, remover }
    }

    pub async fn upload_file(&self, id: Uuid, reader: impl AsyncRead + Unpin) -> Result<(), FileUploadError> {
        let file_upload = self.find_upload_by_id(id).await?;
        let destination_path = self.build_destination_path(&file_upload, &file_upload.name)?;

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

    pub async fn upload_chunk(
        &self,
        id: Uuid,
        index: i64,
        reader: impl AsyncRead + Unpin
    ) -> Result<(), FileUploadError> {
        debug!("upload chunk {} for file {}", index, id);
        let mut file_upload = self.find_upload_by_id(id).await?;

        if file_upload.chunked == false {
            debug!("updating file upload type to chunked");
            file_upload.chunked = true;
            self.save_upload(file_upload.clone()).await?;
        }

        let mut part = Part::new(file_upload.id, index);
        let destination_path = self.build_destination_path(&file_upload, &part.file_name())?;

        // write chunk with 10MB limit
        let bytes_wrote = self.writer.write(reader, destination_path.clone(), Some(1048576)).await?;
        part.size = bytes_wrote;

        let put_part_result = self.put_part_in_manifest(part.clone()).await;
        if let Err(err) = put_part_result {
            if err != ManifestError::DuplicatedPart {
                debug!("removing failed uploaded part: {}", err);
                let remover = self.remover.clone();
                let _ = tokio::task::spawn_blocking(move || remover.remove(destination_path)).await;
            }

            return Err(err.into());
        }

        debug!("checking all parts not exceed file size limit");
        let manifest = self.find_manifest_by_upload_id(id).await?;
        let mut total_bytes = 0;

        for part in manifest.parts {
            total_bytes += part.size;
        }

        if self.limiter.check_file_size(total_bytes) == false {
            return Err(FileUploadError::MaxFileSizeExceeded);
        }

        Ok(())
    }

    async fn save_upload(&self, file_upload: FileUpload) -> Result<(), FileUploadError> {
        let repository = self.repository.clone();
        tokio::task::spawn_blocking(move || repository.save(file_upload)).await
            .map_err(|e| FileUploadError::IO { message: e.to_string() })??;

        Ok(())
    }

    async fn find_upload_by_id(&self, id: Uuid) -> Result<FileUpload, FileUploadError> {
        let repository = self.repository.clone();
        let file_upload = tokio::task::spawn_blocking(move || repository.find_by_id(id)).await
            .map_err(|e| FileUploadError::IO { message: e.to_string() })??;

        if let None = file_upload {
            debug!("file upload {} does not exist", id);
            return Err(FileUploadError::NotExists { id })
        }

        Ok(file_upload.unwrap())
    }

    async fn find_manifest_by_upload_id(&self, id: Uuid) -> Result<Manifest, FileUploadError> {
        let manifest_repo = self.manifest_repository.clone();
        let manifest = tokio::task::spawn_blocking(move || manifest_repo.find_by_upload_id(id)).await
            .map_err(|e| FileUploadError::IO { message: e.to_string() })??;

        Ok(manifest)
    }

    async fn put_part_in_manifest(&self, part: Part) -> Result<(), ManifestError> {
        let manifest_repo = self.manifest_repository.clone();
        let result = tokio::task::spawn_blocking(move || manifest_repo.put_part(part)).await
            .map_err(|err| ManifestError::IO(err.to_string()))?;

        result
    }

    fn build_destination_path(&self, file_upload: &FileUpload, file_name: &String) -> Result<String, FileUploadError> {
        let destination_path = file_upload.directory(&self.settings)?;
        let destination_path = destination_path.join(file_name);
        Ok(destination_path.to_string_lossy().to_string())
    }
}

impl FileUploader<
    PostgresFileUploadRepository,
    SQLiteManifestRepository,
    FileSystemObjectStorageWriter,
    FileSystemObjectStorageRemover
> {
    pub fn get_with_settings(settings: Settings) -> Self {
        Self::new(
            PostgresFileUploadRepository::get_with_settings(settings.clone()),
            SQLiteManifestRepository::new(settings.clone()),
            FileSystemObjectStorageWriter::new(),
            FileSystemObjectStorageRemover::new(),
            settings.clone(),
            FileSizeLimiter::new(settings.clone()),
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
    DB { message: String },
    DuplicatedChunk
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
            Self::DuplicatedChunk => file_upload_codes::DUPLICATED_CHUNK_CODE.to_string(),
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
            Self::DuplicatedChunk => "Chunk is already uploaded".to_string(),
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

impl From<ObjectStorageWriteError> for FileUploadError {
    fn from(value: ObjectStorageWriteError) -> Self {
        match value {
            ObjectStorageWriteError::IO(err) => FileUploadError::IO { message: err.to_string() },
        }
    }
}

impl From<ManifestError> for FileUploadError {
    fn from(value: ManifestError) -> Self {
        match value {
            ManifestError::IO(err) => FileUploadError::IO { message: err },
            ManifestError::DuplicatedPart => FileUploadError::DuplicatedChunk,
        }
    }
}

impl From<ObjectStorageRemoveError> for FileUploadError {
    fn from(value: ObjectStorageRemoveError) -> Self {
        FileUploadError::IO { message: value.to_string() }
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
    pub const DUPLICATED_CHUNK_CODE: &str = "DuplicatedChunk";
}

#[cfg(test)]
mod tests {
    use crate::config::Settings;
    use crate::model::FileUpload;
    use crate::repository::{InMemoryFileUploadRepository, InMemoryManifestRepository};
    use crate::services::object_storage_remover::FakeObjectStorageRemover;
    use crate::services::object_storage_writer::FakeObjectStorageWriter;
    use crate::services::{FileSizeLimiter, FileUploadError, FileUploader};
    use bytes::Bytes;
    use std::collections::HashMap;
    use tokio_util::io::StreamReader;
    use uuid::Uuid;

    impl FileUploader<
        InMemoryFileUploadRepository,
        InMemoryManifestRepository,
        FakeObjectStorageWriter,
        FakeObjectStorageRemover
    > {
        pub fn create() -> Self {
            Self {
                repository: Self::create_repository(),
                manifest_repository: InMemoryManifestRepository::new(),
                writer: FakeObjectStorageWriter,
                settings: Settings::default(),
                limiter: FileSizeLimiter::create(),
                remover: FakeObjectStorageRemover::new()
            }
        }

        pub fn create_with_limit() -> Self {
            Self {
                repository: Self::create_repository(),
                manifest_repository: InMemoryManifestRepository::new(),
                writer: FakeObjectStorageWriter,
                settings: Settings::default(),
                limiter: FileSizeLimiter::create_limited(),
                remover: FakeObjectStorageRemover::new()
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

    #[actix_web::test]
    async fn test_upload_chunk() {
        let uploader = FileUploader::create();
        let index = 0;
        let id = Uuid::parse_str("5769aa43-2380-49be-aafb-e9dd4bd4564f").unwrap();
        let stream = tokio_stream::iter(vec![
            tokio::io::Result::Ok(Bytes::from("Hello")),
            tokio::io::Result::Ok(Bytes::from("World")),
        ]);

        let reader = StreamReader::new(stream);
        let result = uploader.upload_chunk(id, index, reader).await;

        assert_eq!(true, result.is_ok());
    }

    #[actix_web::test]
    async fn test_upload_chunk_not_exists() {
        let uploader = FileUploader::create();
        let index = 0;
        let id = Uuid::parse_str("f5dcca7d-e8ba-4e87-b7b1-e3cde6bc857d").unwrap();
        let stream = tokio_stream::iter(vec![
            tokio::io::Result::Ok(Bytes::from("Hello")),
            tokio::io::Result::Ok(Bytes::from("World")),
        ]);

        let reader = StreamReader::new(stream);
        let result = uploader.upload_chunk(id, index, reader).await;

        assert_eq!(true, result.is_err());
        assert_eq!(FileUploadError::NotExists { id }, result.unwrap_err());
    }

    #[actix_web::test]
    async fn test_upload_chunk_max_file_size_exceeded() {
        let uploader = FileUploader::create_with_limit();
        let index = 0;
        let id = Uuid::parse_str("5769aa43-2380-49be-aafb-e9dd4bd4564f").unwrap();
        let content = vec![1u8; 200];
        let stream = tokio_stream::iter(vec![
            tokio::io::Result::Ok(Bytes::from(content)),
        ]);

        let reader = StreamReader::new(stream);
        let result = uploader.upload_chunk(id, index, reader).await;

        assert_eq!(true, result.is_err());
        assert_eq!(FileUploadError::MaxFileSizeExceeded, result.unwrap_err());
    }
}
