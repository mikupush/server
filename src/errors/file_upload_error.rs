use std::error::Error;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Debug)]
pub enum FileUploadError {
    Exists,
    NotExists { id: Uuid },
    MaxFileSizeExceeded,
    NotCompleted,
    StreamRead { message: String },
    IO { message: String },
    DB { message: String }
}

impl FileUploadError {
    pub fn code(&self) -> String {
        match self {
            FileUploadError::Exists => "Exists".to_string(),
            FileUploadError::NotExists { .. } => "NotExists".to_string(),
            FileUploadError::MaxFileSizeExceeded => "MaxFileSizeExceeded".to_string(),
            FileUploadError::StreamRead { .. } => "StreamRead".to_string(),
            FileUploadError::DB { .. } => "DB".to_string(),
            FileUploadError::IO { .. } => "IO".to_string(),
            FileUploadError::NotCompleted => "NotCompleted".to_string(),
        }
    }
}

impl From<actix_web::error::PayloadError> for FileUploadError {
    fn from(value: actix_web::error::PayloadError) -> Self {
        FileUploadError::StreamRead { message: value.to_string() }
    }
}

impl From<std::io::Error> for FileUploadError {
    fn from(value: std::io::Error) -> Self {
        FileUploadError::IO { message: value.to_string() }
    }
}

impl From<diesel::result::Error> for FileUploadError {
    fn from(value: diesel::result::Error) -> Self {
        FileUploadError::DB { message: value.to_string() }
    }
}

impl From<r2d2::Error> for FileUploadError {
    fn from(value: r2d2::Error) -> Self {
        FileUploadError::DB { message: value.to_string() }
    }
}

impl Error for FileUploadError {}

impl Display for FileUploadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileUploadError::Exists => write!(f, "File is already registered"),
            FileUploadError::NotExists { id: uuid } => write!(f, "File with uuid {} is not registered", uuid),
            FileUploadError::MaxFileSizeExceeded => write!(f, "Max file size exceeded"),
            FileUploadError::StreamRead { message } => write!(f, "Error reading uploaded file stream: {}", message),
            FileUploadError::DB { message } => write!(f, "Database error: {}", message),
            FileUploadError::IO { message } => write!(f, "IO Error: {}", message),
            FileUploadError::NotCompleted => write!(f, "File upload is not completed"),
        }
    }
}
