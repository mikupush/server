// Miku Push! Server is the backend behind Miku Push!
// Copyright (C) 2025  Miku Push! Team
// 
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use actix_files as fs;
use actix_web::{web, App, HttpServer};
use tracing_actix_web::TracingLogger;

mod routes;
mod config;
mod database;
mod schema;
mod serialization;
mod services;
mod errors;
mod logging;
mod repository;
mod model;

use crate::database::setup_database_connection;
use crate::logging::configure_logging;
use crate::routes::json_error_handler;
use clap::Parser;
use config::Settings;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "mikupush-server",
    about = "The Miku Push! Server",
    author = "Miku Push! Team"
)]
struct Cli {
    /// Path to the YAML configuration file
    #[arg(short = 'c', long = "config", value_name = "PATH")]
    config: Option<PathBuf>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let config_path = cli.config;

    Settings::setup_global_from(config_path);
    let settings = Settings::get();

    // logging config
    configure_logging(settings.clone());

    // database connection pool
    let pool = setup_database_connection(&settings);

    // services
    let limiter = services::FileSizeLimiter::new(settings.clone());
    let file_upload_repository = repository::PostgresFileUploadRepository::new(pool.clone());
    let registerer = services::FileRegister::new(file_upload_repository.clone(), limiter.clone());
    let reader = services::FileReader::new(file_upload_repository.clone(), settings.clone());
    let finder = services::FileInfoFinder::new(file_upload_repository.clone(), settings.clone());

    let settings_clone = settings.clone();
    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::JsonConfig::default().error_handler(json_error_handler))
            .app_data(web::Data::new(settings_clone.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(registerer.clone()))
            .app_data(web::Data::new(reader.clone()))
            .app_data(web::Data::new(finder.clone()))
            .service(fs::Files::new("/static", settings_clone.server.static_directory()))
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
