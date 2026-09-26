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

use crate::config::Settings;
use crate::database::DbPool;
use actix_web::http::header::HeaderValue;
use actix_web::{get, web, HttpRequest, HttpResponse, head};
use diesel::{sql_query, RunQueryDsl};
use serde::Deserialize;
use serde_json::json;
use tracing::{debug, warn};
use crate::template::TemplateRenderer;
use crate::tracing::ElapsedTimeTracing;
use crate::serialization::query_string::deserialize_flag;

const ANY_CONTENT_TYPE: &'static str = "*/*";

#[derive(Deserialize)]
struct Options {
    #[serde(default, deserialize_with = "deserialize_flag")]
    json: bool,
}

#[get("/health")]
pub async fn get_health(
    pool: web::Data<DbPool>,
    settings: web::Data<Settings>,
    request: HttpRequest,
    options: web::Query<Options>,
) -> HttpResponse {
    let time_tracing = ElapsedTimeTracing::new("health");
    let default_accept_header = HeaderValue::from_static(ANY_CONTENT_TYPE);
    let accept_header = request.headers().get("Accept")
        .unwrap_or(&default_accept_header)
        .to_str()
        .unwrap_or(ANY_CONTENT_TYPE);
    let json = options.json || accept_header == "application/json";

    if let Err(_) = health_check(&pool).await {
        time_tracing.trace();
        return respond_error(json, &settings, &request).await;
    }

    time_tracing.trace();
    respond_ok(json, &settings, &request).await
}

#[head("/health")]
pub async fn head_health(
    pool: web::Data<DbPool>,
) -> HttpResponse {
    let time_tracing = ElapsedTimeTracing::new("health");

    if let Err(_) = health_check(&pool).await {
        time_tracing.trace();
        return HttpResponse::InternalServerError().finish();
    }

    time_tracing.trace();
    HttpResponse::Ok().finish()
}

async fn health_check(pool: &DbPool) -> Result<(), String> {
    check_db_connection(&pool).await
}

async fn check_db_connection(pool: &DbPool) -> Result<(), String> {
    debug!("checking database connection");
    let connection = pool.get();
    if let Err(err) = connection {
        warn!("unable to get database connection from connection pool: {}", err);
        return Err("database connection failed".to_string());
    }

    let mut connection = connection.unwrap();
    let result = match web::block(move || sql_query("SELECT 1").execute(&mut connection)).await {
        Ok(result) => result,
        Err(err) => return Err(err.to_string())
    };

    if let Err(err) = result {
        warn!("database connection check failed: {}", err);
        return Err("database connection check".to_string());
    }

    Ok(())
}

async fn respond_ok(json: bool, settings: &Settings, request: &HttpRequest) -> HttpResponse {
    if json {
        return HttpResponse::Ok().json(json!({ "status": "up" }))
    }

    let template = TemplateRenderer::new(settings, request);
    let html = template.render("health/ok.html").await;

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn respond_error(json: bool, settings: &Settings, request: &HttpRequest) -> HttpResponse {
    if json {
        return HttpResponse::InternalServerError().json(json!({ "status": "down" }))
    }

    let template = TemplateRenderer::new(settings, request);
    let html = template.render("health/error.html").await;

    HttpResponse::InternalServerError()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::setup_database_connection;
    use crate::routes::utils::tests::header_value;
    use actix_web::http::{Method, StatusCode};
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health_200_ok() {
        let pool = setup_database_connection(&Settings::load(None));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(web::Data::new(Settings::default()))
                .service(get_health)
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
        let pool = setup_database_connection(&Settings::load(None));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(web::Data::new(Settings::default()))
                .service(get_health)
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

    #[actix_web::test]
    async fn test_health_200_ok_json_query_true() {
        let pool = setup_database_connection(&Settings::load(None));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(web::Data::new(Settings::default()))
                .service(get_health)
        ).await;

        let request = test::TestRequest::default()
            .uri("/health?json=true")
            .method(Method::GET)
            .to_request();
        let response = test::call_service(&app, request).await;
        let content_type = header_value("Content-Type", &response);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(content_type, "application/json");
    }

    #[actix_web::test]
    async fn test_health_200_ok_json_query_false() {
        let pool = setup_database_connection(&Settings::load(None));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(web::Data::new(Settings::default()))
                .service(get_health)
        ).await;

        let request = test::TestRequest::default()
            .uri("/health?json=false")
            .method(Method::GET)
            .to_request();
        let response = test::call_service(&app, request).await;
        let content_type = header_value("Content-Type", &response);

        assert_eq!(response.status(), StatusCode::OK);
        assert_ne!(content_type, "application/json");
    }

    #[actix_web::test]
    async fn test_health_200_ok_json_query() {
        let pool = setup_database_connection(&Settings::load(None));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(web::Data::new(Settings::default()))
                .service(get_health)
        ).await;

        let request = test::TestRequest::default()
            .uri("/health?json")
            .method(Method::GET)
            .to_request();
        let response = test::call_service(&app, request).await;
        let content_type = header_value("Content-Type", &response);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(content_type, "application/json");
    }

    #[actix_web::test]
    async fn test_health_200_ok_head() {
        let pool = setup_database_connection(&Settings::load(None));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(web::Data::new(Settings::default()))
                .service(head_health)
        ).await;

        let request = test::TestRequest::default()
            .uri("/health")
            .method(Method::HEAD)
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::OK);
    }
}
