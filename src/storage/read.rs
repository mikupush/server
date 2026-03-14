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

use std::io;
use async_stream::stream;
use async_trait::async_trait;
use bytes::Bytes;
use futures::future::BoxFuture;
use futures::{self, StreamExt};
use futures::stream::BoxStream;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

pub type BoxedStream = BoxStream<'static, io::Result<Bytes>>;
type ResultStream = io::Result<BoxedStream>;

#[async_trait]
pub trait ObjectStorageReader: Send + Sync {
    async fn read(&self, location: String) -> ResultStream;
    fn read_all(&self, location: String) -> BoxFuture<io::Result<Bytes>>;
}

pub struct ObjectStorageReaderFactory;

impl ObjectStorageReaderFactory {
    pub fn get() -> impl ObjectStorageReader + Send + Sync + Clone {
        FileSystemObjectStorageReader::new()
    }
}

#[derive(Debug, Clone)]
pub struct FakeObjectStorageReader;

#[async_trait]
impl ObjectStorageReader for FakeObjectStorageReader {
    async fn read(&self, _location: String) -> ResultStream {
        let data = b"sample content";
        let stream = ReaderStream::new(&data[..]).map(|res| res.map(Bytes::from));
        Ok(stream.boxed())
    }

    fn read_all(&self, _location: String) -> BoxFuture<std::io::Result<Bytes>> {
        Box::pin(async move {
            Ok(Bytes::from("sample content"))
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileSystemObjectStorageReader;

impl FileSystemObjectStorageReader {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ObjectStorageReader for FileSystemObjectStorageReader {
    async fn read(&self, location: String) -> ResultStream {
        let file = File::open(location).await?;
        let stream = ReaderStream::new(file);
        Ok(stream.boxed())
    }

    fn read_all(&self, location: String) -> BoxFuture<std::io::Result<Bytes>> {
        Box::pin(async move {
            tokio::fs::read(location).await.map(Bytes::from)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::TryStreamExt;

    #[tokio::test]
    async fn test_file_system_read() {
        let reader = FileSystemObjectStorageReader::new();

        let content = b"sample content";
        let path = "data/sample_file.txt";
        std::fs::write(path, content).unwrap();

        let result = reader.read(path.to_string()).await;

        assert!(result.is_ok());

        let stream = result.unwrap();
        let stream_content: Vec<u8> = stream
            .map_ok(|b| b.to_vec())
            .try_concat()
            .await
            .unwrap();

        assert_eq!(content, stream_content.as_slice())
    }

    #[tokio::test]
    async fn test_file_system_read_all() {
        let reader = FileSystemObjectStorageReader::new();

        let content = b"sample content";
        let path = "data/sample_file_all.txt";
        std::fs::write(path, content).unwrap();

        let result = reader.read_all(path.to_string()).await;

        assert!(result.is_ok());

        let result_content = result.unwrap().to_vec();
        let result_content = result_content.as_slice();

        assert_eq!(content, result_content)
    }
}
