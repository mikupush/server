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

use std::path::Path;
use actix_web::error::PayloadError;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tracing::debug;

#[cfg(test)]
use crate::config::Upload as UploadSettings;
#[cfg(test)]
use std::path::PathBuf;
use tracing::field::debug;
use crate::tracing::ElapsedTimeTracing;

pub trait ObjectStorageAppender {
    /// Append content to a file.
    /// Returns the total bytes written.
    /// * `destination` - the destination path of the file to append content for example
    async fn append(
        &self,
        reader: impl AsyncRead + Unpin,
        destination: String
    ) -> Result<u64, ObjectStorageAppendError>;
}

#[derive(Clone)]
pub struct FakeObjectStorageAppender;

impl ObjectStorageAppender for FakeObjectStorageAppender {
    async fn append(
        &self,
        mut reader: impl AsyncRead + Unpin,
        _destination: String,
    ) -> Result<u64, ObjectStorageAppendError> {
        Ok(tokio::io::copy(&mut reader, &mut tokio::io::sink()).await?)
    }
}

#[derive(Debug, Clone)]
pub struct FileSystemObjectStorageAppender;

impl FileSystemObjectStorageAppender {
    pub fn new() -> Self {
        Self
    }
}

impl ObjectStorageAppender for FileSystemObjectStorageAppender {
    /// Write an entire file with a size limit.
    async fn append(
        &self,
        mut reader: impl AsyncRead + Unpin,
        destination: String,
    ) -> Result<u64, ObjectStorageAppendError> {
        let time_trace = ElapsedTimeTracing::new("write_file_to_file_system");
        let destination = Path::new(&destination);
        let destination_directory = destination.parent();
        if let Some(destination_directory) = destination_directory
            && destination_directory.exists() == false
        {
            tokio::fs::create_dir_all(destination_directory).await?;
        }

        debug!("append content to {:?}", destination);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&destination)
            .await?;

        let bytes_written = tokio::io::copy(&mut reader, &mut file).await?;
        debug!("wrote {} bytes on {:?}", bytes_written, destination);

        time_trace.trace();
        Ok(bytes_written)
    }
}

#[derive(Debug)]
pub enum ObjectStorageAppendError {
    IO(String),
}

impl From<std::io::Error> for ObjectStorageAppendError {
    fn from(error: std::io::Error) -> Self {
        Self::IO(error.to_string())
    }
}

impl From<PayloadError> for ObjectStorageAppendError {
    fn from(error: PayloadError) -> Self {
        Self::IO(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use bytes::Bytes;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio_util::io::StreamReader;
    use uuid::Uuid;

    impl FileSystemObjectStorageAppender {
        pub fn create() -> Self {
            Self::new()
        }
    }

    #[test]
    async fn test_append_file() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let appender = FileSystemObjectStorageAppender::create();
        let first_content = vec![
            tokio::io::Result::Ok(Bytes::from("Hello")),
            tokio::io::Result::Ok(Bytes::from("World")),
        ];

        let second_content = vec![
            tokio::io::Result::Ok(Bytes::from("KasaneTeto")),
            tokio::io::Result::Ok(Bytes::from("MyBeloved")),
        ];

        let destination = test_directory()
            .join(format!("test_{}.txt", Uuid::new_v4()))
            .to_string_lossy()
            .to_string();

        let reader = StreamReader::new(tokio_stream::iter(first_content));
        appender.append(reader, destination.clone()).await.unwrap();

        let reader = StreamReader::new(tokio_stream::iter(second_content));
        appender.append(reader, destination.clone()).await.unwrap();

        assert_eq!(true, std::fs::exists(&destination).unwrap());
        assert_eq!("HelloWorldKasaneTetoMyBeloved", std::fs::read_to_string(&destination).unwrap());
    }

    fn test_directory() -> PathBuf {
        let directory = PathBuf::from(UploadSettings::default().directory);

        if !std::fs::exists(&directory).unwrap() {
            std::fs::create_dir_all(&directory).unwrap();
        }

        directory
    }
}
