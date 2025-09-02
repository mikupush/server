use actix_web::{delete, web, Responder};
use crate::services::FileDeleter;

#[delete("/api/file/{id}")]
pub async fn delete_file(
    deleter: web::Data<FileDeleter>,
    id: web::Path<String>
) -> impl Responder {
    "Hello world".to_string()
}
