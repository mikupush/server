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

use bytes::Bytes;
use futures::future::BoxFuture;
use futures::{self, StreamExt};
use std::io;
use std::io::{BufReader, Read};
use std::fs::File;

pub type BytesStream = Box<dyn Read + Send + Sync + 'static>;
pub type ResultStream = io::Result<BytesStream>;

pub trait ObjectStorageReader: Send + Sync {
    fn read(&self, location: &String) -> ResultStream;
}

pub struct ObjectStorageReaderFactory;

impl ObjectStorageReaderFactory {
    pub fn get() -> impl ObjectStorageReader + Send + Sync + Clone {
        FileSystemObjectStorageReader::new()
    }
}

#[derive(Debug, Clone)]
pub struct FakeObjectStorageReader;

impl ObjectStorageReader for FakeObjectStorageReader {
    fn read(&self, _location: &String) -> ResultStream {
        let data = b"sample content";
        let stream = BufReader::new(&data[..]);
        Ok(Box::new(stream))
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
    fn read(&self, location: &String) -> ResultStream {
        let file = File::open(location)?;
        Ok(Box::new(file))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_file_system_read() {
        let reader = FileSystemObjectStorageReader::new();

        let content = b"sample content";
        let path = "data/sample_file.txt";
        std::fs::write(path, content).unwrap();

        let result = reader.read(&path.to_string());

        assert!(result.is_ok());

        let mut stream = result.unwrap();
        let mut stream_content = Vec::new();
        stream.read_to_end(&mut stream_content).unwrap();

        assert_eq!(content, stream_content.as_slice())
    }
}
