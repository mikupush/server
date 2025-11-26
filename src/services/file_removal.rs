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

pub struct FileRemovalError(String);

impl Display for FileRemovalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<std::io::Error> for FileRemovalError {
    fn from(err: std::io::Error) -> Self {
        Self(err.to_string())
    }
}

pub trait FileRemoval {
    fn remove(&self, path: String) -> Result<(), FileRemovalError>;
}

#[derive(Debug, Clone)]
pub struct FakeFileRemoval;

impl FakeFileRemoval {
    pub fn new() -> Self {
        Self {}
    }
}

impl FileRemoval for FakeFileRemoval {
    fn remove(&self, path: String) -> Result<(), FileRemovalError> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileSystemFileRemoval;

impl FileSystemFileRemoval {
    pub fn new() -> Self {
        Self {}
    }
}

impl FileRemoval for FileSystemFileRemoval {
    fn remove(&self, path: String) -> Result<(), FileRemovalError> {
        std::fs::remove_file(path)?;
        Ok(())
    }
}