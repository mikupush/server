use crate::errors::Error;
use std::fmt::{Display, Formatter};

pub enum RouteError {
    InvalidPathParameter {
        name: String,
        reason: String,
    },
}

impl Display for RouteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}

impl Error for RouteError {
    fn code(&self) -> String {
        match self {
            Self::InvalidPathParameter { .. } => route_error_codes::INVALID_PATH_PARAMETER_CODE.to_string(),
        }
    }

    fn message(&self) -> String {
        match self {
            Self::InvalidPathParameter { name, reason } => format!("The parameter {} is invalid: {}", name, reason),
        }
    }
}

pub mod route_error_codes {
    pub const INVALID_PATH_PARAMETER_CODE: &'static str = "InvalidPathParameter";
}

pub mod route_error_helpers {
    use crate::errors::RouteError;
    use crate::routes::ErrorResponse;
    use actix_web::HttpResponse;

    pub fn invalid_parameter_response(name: &str, reason: &str) -> HttpResponse {
        let err = RouteError::InvalidPathParameter {
            name: name.to_string(),
            reason: reason.to_string(),
        };

        HttpResponse::BadRequest().json(ErrorResponse::from(err))
    }

    pub fn invalid_uuid(name: &str, value: String) -> HttpResponse {
        invalid_parameter_response(
            name,
            format!("{} is not a valid UUID", value).as_str()
        )
    }
}

