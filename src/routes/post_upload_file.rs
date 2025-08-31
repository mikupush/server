use crate::services::FileUploader;
use actix_web::web::Payload;
use actix_web::{post, web, HttpResponse, Result};
use log::debug;
use uuid::Uuid;
use crate::errors::FileUploadError;

#[post("/api/file/{id}/upload")]
pub async fn post_upload_file(
    file_uploader: web::Data<FileUploader>,
    id: web::Path<String>,
    payload: Payload
) -> Result<HttpResponse> {
    let Ok(id) = Uuid::try_from(id.to_string()) else {
        debug!("cant convert id to uuid: {}", id.to_string());
        return Ok(HttpResponse::BadRequest().finish())
    };

    match file_uploader.upload_file(id, payload).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Ok(handle_post_upload_file_error(err))
    }
}

fn handle_post_upload_file_error(err: FileUploadError) -> HttpResponse {
    match err {
        FileUploadError::NotCompleted => HttpResponse::BadRequest().finish(),
        FileUploadError::MaxFileSizeExceeded => HttpResponse::PayloadTooLarge().finish(),
        FileUploadError::NotExists { .. } => HttpResponse::NotFound().finish(),
        _ => HttpResponse::InternalServerError().finish()
    }
}
