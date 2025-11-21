use crate::config::{Settings, Upload as UploadSettings};
use crate::services::FileSizeLimiter;
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
pub struct NoopFileWriter;

impl FileWriter for NoopFileWriter {
    async fn write(
        &self, _reader:
        impl AsyncRead + Unpin,
        _destination: String,
        _limit: Option<u64>
    ) -> Result<u64, FileWriteError> {
        Ok(0)
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
    FileSizeLimitExceeded
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
            .join(format!("test_{}.txt", now))
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
            .join(format!("test_{}.txt", now))
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
