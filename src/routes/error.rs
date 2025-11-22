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

use crate::errors::{Error, RouteError};
use actix_web::error::JsonPayloadError;
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

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