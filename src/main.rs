/// Copyright 2025 Miku Push! Team
///
/// Licensed under the Apache License, Version 2.0 (the "License");
/// you may not use this file except in compliance with the License.
/// You may obtain a copy of the License at
///
///     http://www.apache.org/licenses/LICENSE-2.0
///
/// Unless required by applicable law or agreed to in writing, software
/// distributed under the License is distributed on an "AS IS" BASIS,
/// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
/// See the License for the specific language governing permissions and
/// limitations under the License.

use actix_web::{web, App, HttpServer};
use actix_files as fs;
use tracing_actix_web::TracingLogger;

mod routes;
mod config;
mod database;
mod model;
mod schema;
mod serialization;
mod services;
mod errors;
mod logging;

use config::Settings;
use crate::database::create_database_connection;
use crate::logging::configure_logging;
use crate::routes::json_error_handler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // app settings
    let settings = Settings::load();
    let settings_clone = settings.clone();

    // logging config
    configure_logging(settings.clone());

    // database connection pool
    let pool = create_database_connection(settings.clone());

    // services
    let limiter = services::FileSizeLimiter::new(settings.clone());
    let registerer = services::FileRegister::new(pool.clone(), limiter.clone());
    let uploader = services::FileUploader::new(pool.clone(), settings.clone(), limiter.clone());
    let deleter = services::FileDeleter::new(pool.clone(), settings.clone());
    let reader = services::FileReader::new(pool.clone(), settings.clone());
    let finder = services::FileInfoFinder::new(pool.clone(), settings.clone());

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::JsonConfig::default().error_handler(json_error_handler))
            .app_data(web::Data::new(settings_clone.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(registerer.clone()))
            .app_data(web::Data::new(uploader.clone()))
            .app_data(web::Data::new(deleter.clone()))
            .app_data(web::Data::new(reader.clone()))
            .app_data(web::Data::new(finder.clone()))
            .service(fs::Files::new("/static", "static"))
            .service(routes::post_file)
            .service(routes::delete_file)
            .service(routes::post_upload_file)
            .service(routes::get_download)
            .service(routes::health)
            .service(routes::get_file_info)
    })
    .bind((settings.server.host(), settings.server.port()))?
    .run()
    .await
}
