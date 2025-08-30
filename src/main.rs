use actix_web::{web, App, HttpServer};

mod routes;
mod service;
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

    let settings = Settings::load();
    let settings_clone = settings.clone();
    let pool = create_database_connection(settings.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(settings_clone.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(services::FileUploadRegister::new(pool.clone(), settings_clone.clone())))
            .service(routes::post_file)
            .service(routes::delete_file)
            .service(routes::post_upload_file)
            .service(routes::get_download)
    })
    .bind((settings.server.host(), settings.server.port()))?
    .run()
    .await
}
