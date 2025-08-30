use crate::errors::FileUploadError;
use crate::services::FileUploadRegister;
use actix_web::{delete, get, post, web, HttpResponse, Responder, Result};
use log::debug;
use serde::Deserialize;
use uuid::Uuid;

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

#[post("/api/file/{uuid}/upload")]
pub async fn post_upload_file(uuid: web::Path<String>) -> impl Responder {
    "Hello world".to_string()
}

#[delete("/api/file/{uuid}")]
pub async fn delete_file(uuid: web::Path<String>) -> impl Responder {
    "Hello world".to_string()
}

#[get("/u/{uuid}")]
pub async fn get_download() -> impl Responder {
    "Hello world".to_string()
}
