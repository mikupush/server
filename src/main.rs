use actix_web::{web, App, HttpServer};

mod routes;
mod service;
mod config;

use config::Settings;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    env_logger::init();

    let settings = Settings::load();
    let settings_clone = settings.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(settings_clone.clone()))
            .service(routes::post_file)
            .service(routes::delete_file)
            .service(routes::post_upload_file)
            .service(routes::get_download)
    })
    .bind((settings.server.host(), settings.server.port()))?
    .run()
    .await
}
