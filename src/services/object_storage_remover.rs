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

use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum ObjectStorageRemoveError {
    IO(String),
    NotExists
}

impl Display for ObjectStorageRemoveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectStorageRemoveError::IO(err) => write!(f, "{}", err),
            ObjectStorageRemoveError::NotExists => write!(f, "file does not exist")
        }
    }
}

impl From<std::io::Error> for ObjectStorageRemoveError {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err.to_string())
    }
}

pub trait ObjectStorageRemover {
    fn remove(&self, location: String) -> Result<(), ObjectStorageRemoveError>;
}

#[derive(Debug, Clone)]
pub struct FakeObjectStorageRemover;

impl FakeObjectStorageRemover {
    pub fn new() -> Self {
        Self {}
    }
}

impl ObjectStorageRemover for FakeObjectStorageRemover {
    fn remove(&self, path: String) -> Result<(), ObjectStorageRemoveError> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileSystemObjectStorageRemover;

impl FileSystemObjectStorageRemover {
    pub fn new() -> Self {
        Self {}
    }
}

impl ObjectStorageRemover for FileSystemObjectStorageRemover {
    /// Remove a file or directory on the local file system.
    /// - `location` - is the path to the file or directory.
    fn remove(&self, location: String) -> Result<(), ObjectStorageRemoveError> {
        if std::fs::exists(&location).unwrap_or(false) == false {
            return Err(ObjectStorageRemoveError::NotExists);
        }

        let is_directory = std::fs::symlink_metadata(&location)
            .map(|m| m.is_dir())
            .unwrap_or(false);

        if is_directory {
            std::fs::remove_dir_all(&location)?;
        } else {
            std::fs::remove_file(&location)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_system_remove_file() {
        let remover = FileSystemObjectStorageRemover::new();
        let path = "data/test.txt";

        std::fs::write(path, "test").unwrap();
        let result = remover.remove(path.to_string());
        let exists = std::fs::exists(path).unwrap_or(false);

        assert!(result.is_ok());
        assert_eq!(false, exists);
    }

    #[test]
    fn test_file_system_remove_directory() {
        let remover = FileSystemObjectStorageRemover::new();
        let path = "data/example-directory";

        std::fs::create_dir_all("data/example-directory").unwrap();
        let result = remover.remove(path.to_string());
        let exists = std::fs::exists(path).unwrap_or(false);

        assert!(result.is_ok());
        assert_eq!(false, exists);
    }

    #[test]
    fn test_file_system_remove_not_existing_object() {
        let remover = FileSystemObjectStorageRemover::new();
        let path = "data/example-directory-not-existing";

        let result = remover.remove(path.to_string());

        assert!(result.is_err());
        assert_eq!(ObjectStorageRemoveError::NotExists, result.unwrap_err());
    }
}
