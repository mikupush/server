use serde::{Deserialize, Serialize};
use crate::errors::FileUploadError;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl From<FileUploadError> for ErrorResponse {
    fn from(value: FileUploadError) -> Self {
        Self {
            code: value.code(),
            message: value.to_string(),
        }
    }
}
