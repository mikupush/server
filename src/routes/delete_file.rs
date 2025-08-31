use actix_web::{delete, web, Responder};

#[delete("/api/file/{uuid}")]
pub async fn delete_file(uuid: web::Path<String>) -> impl Responder {
    "Hello world".to_string()
}
