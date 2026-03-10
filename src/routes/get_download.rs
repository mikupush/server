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
use crate::errors::{route_error_helpers, FileReadError};
use crate::routes::ErrorResponse;
use crate::services::{FileReader, SingleFileReader};
use crate::template::TemplateRenderer;
use crate::tracing::ElapsedTimeTracing;
use actix_http::header::{QualityItem, USER_AGENT};
use actix_web::http::header::Accept;
use actix_web::{get, mime, web, HttpRequest, HttpResponse};
use serde_json::json;
use tracing::debug;
use uuid::Uuid;

#[get("/u/{id}")]
pub async fn get_download(
    settings: web::Data<Settings>,
    id: web::Path<String>,
    accept: Option<web::Header<Accept>>,
    request: HttpRequest
) -> HttpResponse {
    let time_tracing = ElapsedTimeTracing::new("get_download");

    let Ok(id) = Uuid::try_from(id.to_string()) else {
        debug!("cant convert id to uuid: {}", id.to_string());
        return route_error_helpers::invalid_uuid("id", id.to_string())
    };

    let user_agent = request.headers()
        .get(USER_AGENT)
        .and_then(|user_agent| user_agent.to_str().ok())
        .unwrap_or("");

    let accept = accept
        .map(|value| value.0)
        .unwrap_or(Accept::star());
    // TODO: aislar solo a contenido multimedia
    // TODO: cachear query a postgres (15s)
    // TODO: llamar al servicio de FileInfo para saber que tipo es
    let should_show_download_page =
        !user_agent.contains("Discordbot")
        && accept.contains(&QualityItem::max(mime::TEXT_HTML));

    let response = if should_show_download_page {
        respond_download_page(&id, &settings, &request).await
    } else {
        respond_raw_content(&id, &settings).await
    };

    time_tracing.trace();
    response
}

async fn respond_download_page(id: &Uuid, settings: &Settings, request: &HttpRequest) -> HttpResponse {
    let mut template_renderer = TemplateRenderer::new(settings, request);
    template_renderer.add_to_head(format!(
        "<script id=\"upload-metadata\" type=\"application/json\">{}</script>",
        json!({"id": id.to_string()})
    ));
    let html = template_renderer.render_localized("download.html").await;

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn respond_raw_content(id: &Uuid, settings: &Settings) -> HttpResponse {
    let file_reader = FileReader::get_with_settings(settings);
    let result = file_reader.read(*id).await;

    match result {
        Ok(stream_wrapper) => {
            HttpResponse::Ok()
                .content_type(stream_wrapper.details.mime_type.clone())
                .insert_header(("Content-Length", stream_wrapper.details.size.to_string()))
                .insert_header(("Content-Disposition", format!("inline; filename=\"{}\"", stream_wrapper.details.name)))
                .insert_header(("Content-Type", stream_wrapper.details.mime_type.to_string()))
                .streaming(stream_wrapper.stream)
        },
        Err(err) => {
            let mut response_builder = match err {
                FileReadError::NotExists { .. } => HttpResponse::NotFound(),
                _ => HttpResponse::InternalServerError()
            };

            response_builder.json(ErrorResponse::from(err))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use crate::database::setup_database_connection;
    use crate::errors::{file_read_codes, route_error_codes};
    use crate::routes::utils::tests::{create_test_chunked_file_upload, create_test_file_upload, header_value};
    use actix_web::http::{Method, StatusCode};
    use actix_web::{test, App};
    use serial_test::serial;

    #[actix_web::test]
    #[serial]
    async fn test_get_download_200_ok() {
        let settings = Settings::load(None);
        let pool = setup_database_connection(&settings);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(get_download)
        ).await;

        let (_, file_upload) = create_test_file_upload(pool.clone());
        let request = test::TestRequest::default()
            .uri(format!("/u/{}", file_upload.id.clone()).as_str())
            .method(Method::GET)
            .to_request();
        let response = test::call_service(&app, request).await;
        let content_length = header_value("Content-Length", &response);
        let content_disposition = header_value("Content-Disposition", &response);
        let content_type = header_value("Content-Type", &response);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(content_length, file_upload.size.to_string());
        assert_eq!(content_disposition, format!("inline; filename=\"{}\"", file_upload.name));
        assert_eq!(content_type, file_upload.mime_type);
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_download_200_chunked_ok() {
        let settings = Settings::load(None);
        let pool = setup_database_connection(&settings);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings.clone()))
                .service(get_download)
        ).await;

        let (_, file_upload) = create_test_chunked_file_upload(&pool, &settings);
        let request = test::TestRequest::default()
            .uri(format!("/u/{}", file_upload.id.clone()).as_str())
            .method(Method::GET)
            .to_request();
        let response = test::call_service(&app, request).await;
        let content_length = header_value("Content-Length", &response);
        let content_disposition = header_value("Content-Disposition", &response);
        let content_type = header_value("Content-Type", &response);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(content_length, file_upload.size.to_string());
        assert_eq!(content_disposition, format!("inline; filename=\"{}\"", file_upload.name));
        assert_eq!(content_type, file_upload.mime_type);

        let body = test::read_body(response).await;
        assert_eq!(body, "HelloWorld");
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_download_404_not_found() {
        let settings = Settings::load(None);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(get_download)
        ).await;

        let id = Uuid::new_v4();
        let request = test::TestRequest::default()
            .uri(format!("/u/{}", id.clone()).as_str())
            .method(Method::GET)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        let response_body = test::read_body(response).await;

        assert_eq!(status_code, StatusCode::NOT_FOUND);

        let response_body = serde_json::from_slice::<ErrorResponse>(&response_body).unwrap();
        assert_eq!(response_body.code, file_read_codes::NOT_EXISTS_CODE);
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_download_400_bad_request_invalid_id() {
        let settings = Settings::load(None);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(get_download)
        ).await;

        let id = "invalid_uuid";
        let request = test::TestRequest::default()
            .uri(format!("/u/{}", id).as_str())
            .method(Method::GET)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        let response_body = test::read_body(response).await;

        assert_eq!(status_code, StatusCode::BAD_REQUEST);

        let response_body = serde_json::from_slice::<ErrorResponse>(&response_body).unwrap();
        assert_eq!(response_body.code, route_error_codes::INVALID_PATH_PARAMETER_CODE);
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_download_200_html_ok() {
        let settings = Settings::load(None);
        let pool = setup_database_connection(&settings);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(get_download)
        ).await;

        let (_, file_upload) = create_test_file_upload(pool.clone());
        let request = test::TestRequest::default()
            .uri(format!("/u/{}", file_upload.id.clone()).as_str())
            .method(Method::GET)
            .insert_header(("Accept", "text/html"))
            .to_request();
        let response = test::call_service(&app, request).await;
        let content_type = header_value("Content-Type", &response);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(content_type, "text/html; charset=utf-8");
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_download_200_raw_ok_when_discord() {
        let settings = Settings::load(None);
        let pool = setup_database_connection(&settings);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(get_download)
        ).await;

        let (_, file_upload) = create_test_file_upload(pool.clone());
        let request = test::TestRequest::default()
            .uri(format!("/u/{}", file_upload.id.clone()).as_str())
            .method(Method::GET)
            .insert_header(("Accept", "text/html"))
            .insert_header(("User-Agent", "Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com)"))
            .to_request();
        let response = test::call_service(&app, request).await;
        let content_type = header_value("Content-Type", &response);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(content_type, file_upload.mime_type);
    }
}
