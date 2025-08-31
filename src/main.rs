use actix_web::{web, App, HttpServer};

mod routes;
mod config;
mod database;
mod model;
mod schema;
mod serialization;
mod services;
mod errors;

use config::Settings;
use crate::database::create_database_connection;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    env_logger::init();

    // app settings
    let settings = Settings::load();
    let settings_clone = settings.clone();

    // database connection pool
    let pool = create_database_connection(settings.clone());

    // services
    let limiter = services::FileSizeLimiter::new(settings.clone());
    let registerer = services::FileRegister::new(pool.clone(), limiter.clone());
    let uploader = services::FileUploader::new(pool.clone(), limiter.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(settings_clone.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(registerer.clone()))
            .app_data(web::Data::new(uploader.clone()))
            .service(routes::post_file)
            .service(routes::delete_file)
            .service(routes::post_upload_file)
            .service(routes::get_download)
    })
    .bind((settings.server.host(), settings.server.port()))?
    .run()
    .await
}
