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
use std::io::Read;
use std::path::Path;
use std::fs::OpenOptions;
use tracing::debug;
use crate::tracing::ElapsedTimeTracing;

pub trait ObjectStorageAppender {
    /// Append content to a file.
    /// Returns the total bytes written.
    /// * `destination` - the destination path of the file to append content for example
    fn append(
        &self,
        reader: impl Read + Unpin,
        destination: String
    ) -> Result<u64, ObjectStorageAppendError>;
}

#[derive(Clone)]
pub struct FakeObjectStorageAppender;

impl ObjectStorageAppender for FakeObjectStorageAppender {
    fn append(
        &self,
        mut reader: impl Read + Unpin,
        _destination: String,
    ) -> Result<u64, ObjectStorageAppendError> {
        Ok(io::copy(&mut reader, &mut io::sink())?)
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
    fn append(
        &self,
        mut reader: impl Read + Unpin,
        destination: String,
    ) -> Result<u64, ObjectStorageAppendError> {
        let time_trace = ElapsedTimeTracing::new("append_file_to_file_system");
        let destination = Path::new(&destination);
        let destination_directory = destination.parent();
        if let Some(destination_directory) = destination_directory
            && destination_directory.exists() == false
        {
            std::fs::create_dir_all(destination_directory)?;
        }

        debug!("append content to {:?}", destination);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&destination)?;

        let bytes_written = io::copy(&mut reader, &mut file)?;
        debug!("wrote {} bytes on {:?}", bytes_written, destination);

        time_trace.trace();
        Ok(bytes_written)
    }
}

#[derive(Debug)]
pub enum ObjectStorageAppendError {
    IO(String),
}

impl From<io::Error> for ObjectStorageAppendError {
    fn from(error: io::Error) -> Self {
        Self::IO(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use super::*;
    use uuid::Uuid;
    use crate::config::Settings;

    impl FileSystemObjectStorageAppender {
        pub fn create() -> Self {
            Self::new()
        }
    }

    #[test]
    fn test_append_file() {
        let appender = FileSystemObjectStorageAppender::create();
        let destination = test_directory()
            .join(format!("test_{}.txt", Uuid::new_v4()))
            .to_string_lossy()
            .to_string();

        let content = b"HelloWorld";
        let reader = io::BufReader::new(&content[..]);
        appender.append(reader, destination.clone()).unwrap();

        let content = b"KasaneTetoMyBeloved";
        let reader = io::BufReader::new(&content[..]);
        appender.append(reader, destination.clone()).unwrap();

        assert_eq!(true, std::fs::exists(&destination).unwrap());
        assert_eq!("HelloWorldKasaneTetoMyBeloved", std::fs::read_to_string(&destination).unwrap());
    }

    fn test_directory() -> PathBuf {
        let directory = PathBuf::from(Settings::default().upload.directory);

        if !std::fs::exists(&directory).unwrap() {
            std::fs::create_dir_all(&directory).unwrap();
        }

        directory
    }
}
