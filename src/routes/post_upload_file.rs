use actix_web::{post, web, Responder};

#[post("/api/file/{uuid}/upload")]
pub async fn post_upload_file(uuid: web::Path<String>) -> impl Responder {
    "Hello world".to_string()
}
