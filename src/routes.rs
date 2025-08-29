use actix_web::{delete, get, post, web, Responder};

#[post("/api/file")]
pub async fn post_file() -> String {
    "Hello world".to_string()
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
