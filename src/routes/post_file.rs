use actix_web::{post, web, HttpResponse, Result};
use uuid::Uuid;
use log::debug;
use serde::Deserialize;
use crate::errors::FileUploadError;
use crate::services::FileUploadRegister;

#[derive(Debug, Clone, Deserialize)]
pub struct FileCreate {
    #[serde(with = "crate::serialization::uuid")]
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
}

#[post("/api/file")]
pub async fn post_file(
    request: web::Json<FileCreate>,
    file_upload_register: web::Data<FileUploadRegister>
) -> Result<HttpResponse> {
    let request = request.into_inner();

    match file_upload_register.register_file(request.clone()) {
        Ok(_) => {
            debug!("returning status code 200 for registered file {}", request.id);
            Ok(HttpResponse::Ok().finish())
        },
        Err(err) => Ok(handle_register_file_failure(request, err))
    }
}

fn handle_register_file_failure(request: FileCreate, err: FileUploadError) -> HttpResponse {
    debug!("handling register file error: {}: {}", request.id, err);

    let response = match err {
        FileUploadError::Exists => HttpResponse::Conflict().finish(),
        FileUploadError::MaxFileSizeExceeded => HttpResponse::PayloadTooLarge().finish(),
        _ => HttpResponse::InternalServerError().finish()
    };

    debug!("returning error status code {} for file register with id {}", response.status(), request.id);
    response
}
