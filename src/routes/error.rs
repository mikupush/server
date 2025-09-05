use actix_web::error::JsonPayloadError;
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use crate::errors::{Error, RouteError};

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

pub fn json_error_handler(err: JsonPayloadError, _req: &actix_web::HttpRequest) -> actix_web::Error {
    let response = match &err {
        JsonPayloadError::ContentType => {
            let error = RouteError::UnsupportedContentType {
                desired_content_type: "application/json".to_string()
            };

            HttpResponse::UnsupportedMediaType().json(ErrorResponse::from(error))
        }
        _ => {
            let error = RouteError::InvalidRequestBody;
            HttpResponse::BadRequest().json(ErrorResponse::from(error))
        }
    };

    actix_web::error::InternalError::from_response(err, response).into()
}
