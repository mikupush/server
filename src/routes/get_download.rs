use actix_web::{get, Responder};

#[get("/u/{uuid}")]
pub async fn get_download() -> impl Responder {
    "Hello world".to_string()
}
