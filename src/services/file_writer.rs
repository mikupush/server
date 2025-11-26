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

use crate::config::Upload as UploadSettings;
use actix_web::error::PayloadError;
use futures::{StreamExt, TryFutureExt};
use std::io::Write;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;

pub trait FileWriter {
    /// Write an entire file with an optional size limit.
    /// Returns the total bytes written.
    /// * `limit` - size limit in bytes (optional)
    async fn write(
        &self,
        reader: impl AsyncRead + Unpin,
        destination: String,
        limit: Option<u64>
    ) -> Result<u64, FileWriteError>;
}

#[derive(Clone)]
pub struct FakeFileWriter;

impl FileWriter for FakeFileWriter {
    async fn write(
        &self,
        mut reader: impl AsyncRead + Unpin,
        _destination: String,
        _limit: Option<u64>
    ) -> Result<u64, FileWriteError> {
        Ok(tokio::io::copy(&mut reader, &mut tokio::io::sink()).await?)
    }
}

#[derive(Debug, Clone)]
pub struct FileSystemFileWriter;

impl FileSystemFileWriter {
    pub fn new() -> Self {
        Self
    }
}

impl FileWriter for FileSystemFileWriter {
    /// Write an entire file with a size limit.
    async fn write(
        &self,
        mut reader: impl AsyncRead + Unpin,
        destination: String,
        limit: Option<u64>
    ) -> Result<u64, FileWriteError> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&destination)
            .await?;

        let bytes_written = if let Some(limit) = limit {
            let mut limited_reader = reader.take(limit + 1);
            tokio::io::copy(&mut limited_reader, &mut file).await?
        } else {
            tokio::io::copy(&mut reader, &mut file).await?
        };

        Ok(bytes_written)
    }
}

#[derive(Debug)]
pub enum FileWriteError {
    Io(String),
}

impl From<std::io::Error> for FileWriteError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<PayloadError> for FileWriteError {
    fn from(error: PayloadError) -> Self {
        Self::Io(error.to_string())
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

    impl FileSystemFileWriter {
        pub fn create() -> Self {
            Self::new()
        }
    }

    #[test]
    async fn test_write_file() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let writer = FileSystemFileWriter::create();
        let stream = tokio_stream::iter(vec![
            tokio::io::Result::Ok(Bytes::from("Hello")),
            tokio::io::Result::Ok(Bytes::from("World")),
        ]);

        let reader = StreamReader::new(stream);
        let destination = test_directory()
            .join(format!("test_{}.txt", Uuid::new_v4()))
            .to_string_lossy()
            .to_string();

        writer.write(reader, destination.clone(), None).await.unwrap();

        assert_eq!(true, std::fs::exists(&destination).unwrap());
        assert_eq!("HelloWorld", std::fs::read_to_string(&destination).unwrap());
    }

    #[test]
    async fn test_write_file_limited() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let writer = FileSystemFileWriter::create();
        let stream = tokio_stream::iter(vec![
            tokio::io::Result::Ok(Bytes::from("Hello")),
            tokio::io::Result::Ok(Bytes::from("World")),
        ]);

        let reader = StreamReader::new(stream);
        let destination = test_directory()
            .join(format!("test_{}.txt", Uuid::new_v4()))
            .to_string_lossy()
            .to_string();

        writer.write(reader, destination.clone(), Some(1)).await.unwrap();

        assert_eq!(true, std::fs::exists(&destination).unwrap());
        assert_eq!("He", std::fs::read_to_string(&destination).unwrap());
    }

    fn test_directory() -> PathBuf {
        let directory = PathBuf::from(UploadSettings::create().directory());

        if !std::fs::exists(&directory).unwrap() {
            std::fs::create_dir_all(&directory).unwrap();
        }

        directory
    }
}