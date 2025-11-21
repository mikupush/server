use crate::config::Upload as UploadSettings;
use crate::services::FileSizeLimiter;
use actix_web::error::PayloadError;
use actix_web::web::Payload;
use futures::{StreamExt, TryFutureExt};
use std::io::Write;
use std::path::PathBuf;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tracing::debug;

pub trait FileWriter {
    async fn write(&self, reader: impl AsyncRead + Unpin, destination: String) -> Result<(), FileWriteError>;
    async fn write_chunk(&self, reader: impl AsyncRead + Unpin, destination: String) -> Result<(), FileWriteError>;
}

#[derive(Debug, Clone)]
pub struct FileSystemFileWriter {
    settings: UploadSettings,
    limiter: FileSizeLimiter
}

impl FileSystemFileWriter {
    pub fn new(settings: UploadSettings, limiter: FileSizeLimiter) -> Self {
        Self { settings, limiter }
    }
}

impl FileWriter for FileSystemFileWriter {
    /// Write an entire file with a size limit.
    async fn write(&self, mut reader: impl AsyncRead + Unpin, destination: String) -> Result<(), FileWriteError> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&destination)
            .await?;
        let mut bytes_written = 0;

        if self.settings.is_limited() {
            let mut limited_reader = reader.take(self.settings.max_size().unwrap() + 1);
            bytes_written = tokio::io::copy(&mut limited_reader, &mut file).await?;
        } else {
            bytes_written = tokio::io::copy(&mut reader, &mut file).await?;
        }

        if self.limiter.check_file_size(bytes_written) == false {
            debug!("file size limit exceeded");
            return Err(FileWriteError::FileSizeLimitExceeded);
        }

        Ok(())
    }

    /// Write a chunk of 10MB of data.
    async fn write_chunk(&self, reader: impl AsyncRead + Unpin, destination: String) -> Result<(), FileWriteError> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&destination)
            .await?;

        let mut limited_reader = reader.take(10485760); // 10 MB
        tokio::io::copy(&mut limited_reader, &mut file).await?;

        Ok(())
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
    use tokio_util::io::StreamReader;
    use std::time::{SystemTime, UNIX_EPOCH};

    impl FileSystemFileWriter {
        pub fn create() -> Self {
            Self::new(
                UploadSettings::create(),
                FileSizeLimiter::create_unlimited()
            )
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

        writer.write(reader, destination.clone()).await.unwrap();

        assert_eq!(true, std::fs::exists(&destination).unwrap());
        assert_eq!("HelloWorld", std::fs::read_to_string(&destination).unwrap());
    }

    #[test]
    async fn test_write_file_chunk() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let writer = FileSystemFileWriter::create();
        let stream = tokio_stream::iter(vec![
            tokio::io::Result::Ok(Bytes::from("Hello")),
        ]);

        let reader = StreamReader::new(stream);
        let destination = test_directory()
            .join(format!("test_{}.part", now))
            .to_string_lossy()
            .to_string();

        writer.write_chunk(reader, destination.clone()).await.unwrap();

        assert_eq!(true, std::fs::exists(&destination).unwrap());
        assert_eq!("Hello", std::fs::read_to_string(&destination).unwrap());
    }

    fn test_directory() -> PathBuf {
        let directory = PathBuf::from(UploadSettings::create().directory());

        if !std::fs::exists(&directory).unwrap() {
            std::fs::create_dir_all(&directory).unwrap();
        }

        directory
    }
}
