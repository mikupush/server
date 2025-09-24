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

use crate::database::DbPool;
use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_web::http::header::HeaderValue;
use diesel::{sql_query, RunQueryDsl};
use log::{debug, warn};
use serde_json::json;

const ANY_CONTENT_TYPE: &'static str = "*/*";

#[get("/health")]
pub async fn health(
    pool: web::Data<DbPool>,
    request: HttpRequest
) -> HttpResponse {
    let default_accept_header = HeaderValue::from_static(ANY_CONTENT_TYPE);
    let accept_header = request.headers().get("Accept")
        .unwrap_or(&default_accept_header)
        .to_str()
        .unwrap_or(ANY_CONTENT_TYPE);
    let json = accept_header == "application/json";

    if let Err(_) = check_db_connection(&pool) {
        return respond_error(json);
    }

    respond_ok(json)
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

fn respond_ok(json: bool) -> HttpResponse {
    if json {
        return HttpResponse::Ok().json(json!({ "status": "up" }))
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(std::fs::read_to_string("templates/health_ok.html").unwrap())
}

fn respond_error(json: bool) -> HttpResponse {
    if json {
        return HttpResponse::InternalServerError().json(json!({ "status": "down" }))
    }

    HttpResponse::InternalServerError()
        .content_type("text/html; charset=utf-8")
        .body(std::fs::read_to_string("templates/health_error.html").unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::tests::create_test_database_connection;
    use crate::routes::utils::tests::header_value;
    use actix_web::http::{Method, StatusCode};
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health_200_ok() {
        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
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
        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
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
