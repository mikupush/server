use bytes::Bytes;
use futures::future::BoxFuture;
use futures::Stream;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

pub trait ObjectStorageReader {
    fn read(&self, location: String) -> BoxFuture<std::io::Result<impl Stream<Item = std::io::Result<Bytes>> + Send + Unpin + 'static>>;
    fn read_all(&self, location: String) -> BoxFuture<std::io::Result<Bytes>>;
}

#[derive(Debug, Clone)]
pub struct FakeObjectStorageReader;

impl ObjectStorageReader for FakeObjectStorageReader {
    fn read(&self, _location: String) -> BoxFuture<std::io::Result<impl Stream<Item = std::io::Result<Bytes>> + Send + Unpin + 'static>> {
        Box::pin(async move {
            let data = b"sample content";
            let stream = ReaderStream::new(&data[..]);
            Ok(stream)
        })
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

impl ObjectStorageReader for FileSystemObjectStorageReader {
    fn read(&self, location: String) -> BoxFuture<std::io::Result<impl Stream<Item = std::io::Result<Bytes>> + Send + Unpin + 'static>> {
        Box::pin(async move {
            let file = File::open(location).await?;
            let stream = ReaderStream::new(file);
            Ok(stream)
        })
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
