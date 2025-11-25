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
