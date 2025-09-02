use serde::{Deserialize, Serialize};
use crate::errors::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn from(value: impl Error) -> Self {
        Self {
            code: value.code(),
            message: value.to_string(),
        }
    }
}
