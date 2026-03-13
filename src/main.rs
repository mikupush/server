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
mod file;
mod errors;
mod logging;
mod model;
mod tracing;
mod jobs;
mod template;
mod cache;
mod storage;
mod clock;

use crate::database::setup_database_connection;
use crate::logging::configure_logging;
use crate::routes::json_error_handler;
use clap::Parser;
use config::Settings;
use std::path::PathBuf;
use std::time::Duration;
use actix_web::middleware::DefaultHeaders;

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

    let settings = Settings::setup_global_from(config_path);

    // logging config
    let _guard = configure_logging(&settings);

    // database connection pool
    let pool = setup_database_connection(&settings);

    // launch scheduled jobs
    jobs::start_cleanup_expired_files(settings.clone());

    let settings_clone = settings.clone();
    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::JsonConfig::default().error_handler(json_error_handler))
            .app_data(web::Data::new(settings_clone.clone()))
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope(&settings.server.static_base_path)
                    .wrap(DefaultHeaders::new().add(("Cache-Control", "public, max-age=31536000")))
                    .service(fs::Files::new("/", settings.server.static_directory.clone())
                        .use_etag(true)
                        .use_last_modified(true))
            )
            .service(routes::post_file)
            .service(routes::delete_file)
            .service(routes::post_upload_file)
            .service(routes::post_upload_part)
            .service(routes::get_download)
            .service(routes::health)
            .service(routes::get_file_info)
    })
    .keep_alive(Duration::from_secs(10))
    .bind((settings.server.host, settings.server.port))?
    .run()
    .await
}
