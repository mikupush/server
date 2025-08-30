use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum FileUploadError {
    Exists,
    MaxFileSizeExceeded,
    Other { message: String }
}

impl Error for FileUploadError {}

impl Display for FileUploadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileUploadError::Exists => write!(f, "File is already registered"),
            FileUploadError::MaxFileSizeExceeded => write!(f, "Max file size exceeded"),
            FileUploadError::Other { message } => write!(f, "{}", message),
        }
    }
}
