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

use crate::database::DbPool;
use crate::config::Settings;
use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_web::http::header::HeaderValue;
use diesel::{sql_query, RunQueryDsl};
use tracing::{debug, warn};
use serde_json::json;
use crate::routes::utils::read_template;

const ANY_CONTENT_TYPE: &'static str = "*/*";

#[get("/health")]
pub async fn health(
    pool: web::Data<DbPool>,
    settings: web::Data<Settings>,
    request: HttpRequest
) -> HttpResponse {
    let default_accept_header = HeaderValue::from_static(ANY_CONTENT_TYPE);
    let accept_header = request.headers().get("Accept")
        .unwrap_or(&default_accept_header)
        .to_str()
        .unwrap_or(ANY_CONTENT_TYPE);
    let json = accept_header == "application/json";

    if let Err(_) = check_db_connection(&pool) {
        return respond_error(json, &settings);
    }

    respond_ok(json, &settings)
}

fn check_db_connection(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    debug!("checking database connection");
    let connection = pool.get();
    if let Err(err) = connection {
        warn!("unable to get database connection from connection pool: {}", err);
        return Err(err.into());
    }

    let mut connection = connection.unwrap();
    let result = sql_query("SELECT 1").execute(&mut connection);
    if let Err(err) = result {
        warn!("database connection check failed: {}", err);
        return Err(err.into());
    }

    Ok(())
}

fn respond_ok(json: bool, settings: &Settings) -> HttpResponse {
    if json {
        return HttpResponse::Ok().json(json!({ "status": "up" }))
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(read_template(settings, "health_ok.html"))
}

fn respond_error(json: bool, settings: &Settings) -> HttpResponse {
    if json {
        return HttpResponse::InternalServerError().json(json!({ "status": "down" }))
    }

    HttpResponse::InternalServerError()
        .content_type("text/html; charset=utf-8")
        .body(read_template(settings, "health_error.html"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::tests::get_test_database_connection;
    use crate::routes::utils::tests::header_value;
    use actix_web::http::{Method, StatusCode};
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health_200_ok() {
        let pool = get_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(web::Data::new(Settings::default()))
                .service(health)
        ).await;

        let request = test::TestRequest::default()
            .uri("/health")
            .method(Method::GET)
            .to_request();
        let response = test::call_service(&app, request).await;
        let content_type = header_value("Content-Type", &response);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(content_type, "text/html; charset=utf-8");
    }

    #[actix_web::test]
    async fn test_health_200_ok_json() {
        let pool = get_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(web::Data::new(Settings::default()))
                .service(health)
        ).await;

        let request = test::TestRequest::default()
            .uri("/health")
            .method(Method::GET)
            .insert_header(("Accept", "application/json"))
            .to_request();
        let response = test::call_service(&app, request).await;
        let content_type = header_value("Content-Type", &response);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(content_type, "application/json");
    }
}
