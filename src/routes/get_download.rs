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

use actix_http::header::{Header, QualityItem};
use crate::config::Settings;
use crate::file::error::{Error, FileInfoError, FileReadError};
use crate::routes::ErrorResponse;
use crate::file::{FileInfoFinder, FileReader, SingleFileReader};
use crate::template::TemplateRenderer;
use crate::tracing::ElapsedTimeTracing;
use actix_http::StatusCode;
use actix_web::{get, mime, web, HttpRequest, HttpResponse};
use actix_web::http::header::{Accept, ACCEPT, USER_AGENT};
use log::warn;
use serde_json::json;
use tracing::debug;
use uuid::Uuid;
use crate::routes::error::helper;

#[get("/u/{id}")]
pub async fn get_download(
    settings: web::Data<Settings>,
    id: web::Path<String>,
    request: HttpRequest
) -> HttpResponse {
    let time_tracing = ElapsedTimeTracing::new("get_download");
    let finder = FileInfoFinder::get_with_settings(&settings);
    let Ok(id) = Uuid::try_from(id.to_string()) else {
        debug!("cant convert id to uuid: {}", id.to_string());
        return helper::invalid_uuid("id", id.to_string())
    };

    let error_responder = ErrorResponder::new(&settings, &request);
    let file_upload = match finder.find(&id) {
        Ok(file_upload) => file_upload,
        Err(err) => return match err {
            FileInfoError::NotExists { .. } => error_responder.respond(err, StatusCode::NOT_FOUND).await,
            _ => error_responder.respond(err, StatusCode::INTERNAL_SERVER_ERROR).await
        }
    };

    let success_responder = Responder::new(&id, &file_upload.mime_type, &settings, &request);
    let response = success_responder.respond().await;
    time_tracing.trace();

    response
}

struct Responder {
    request: HttpRequest,
    settings: Settings,
    id: Uuid,
    mime_type: String,
}

impl Responder {
    pub fn new(
        id: &Uuid,
        mime_type: &String,
        settings: &Settings,
        request: &HttpRequest
    ) -> Self {
        Self {
            id: id.clone(),
            mime_type: mime_type.clone(),
            settings: settings.clone(),
            request: request.clone()
        }
    }

    pub fn accept_html(request: &HttpRequest) -> bool {
        let accept = Accept::parse(request).ok()
            .unwrap_or(Accept::star());

        accept.contains(&QualityItem::max(mime::TEXT_HTML))
    }

    fn is_robot_request(&self) -> bool {
        let user_agent = self.request.headers()
            .get(USER_AGENT)
            .and_then(|user_agent| user_agent.to_str().ok())
            .unwrap_or("");

        user_agent.contains("Discordbot")
    }

    fn is_crawlable(&self) -> bool {
        self.mime_type.starts_with("video/")
        || self.mime_type.starts_with("audio/")
        || self.mime_type.starts_with("image/")
    }

    pub async fn respond(&self) -> HttpResponse {
        if self.is_robot_request() && self.is_crawlable() {
            return self.raw().await
        }

        if Responder::accept_html(&self.request) {
            self.html().await
        } else {
            self.raw().await
        }
    }

    async fn html(&self) -> HttpResponse {
        let mut template_renderer = TemplateRenderer::new(&self.settings, &self.request);
        template_renderer.add_to_head(format!(
            "<script id=\"upload-metadata\" type=\"application/json\">{}</script>",
            json!({"id": self.id.to_string()})
        ));
        let html = template_renderer.render_localized("download.html").await;

        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)
    }

    async fn raw(&self) -> HttpResponse {
        let file_reader = FileReader::get_with_settings(&self.settings);
        let stream_wrapper = match file_reader.read(self.id).await {
            Ok(stream_wrapper) => stream_wrapper,
            Err(err) => {
                warn!("unable to respond with file contents: {}", err);
                return HttpResponse::InternalServerError().finish();
            }
        };

        HttpResponse::Ok()
            .content_type(stream_wrapper.details.mime_type.clone())
            .insert_header(("Content-Length", stream_wrapper.details.size.to_string()))
            .insert_header(("Content-Disposition", format!("inline; filename=\"{}\"", stream_wrapper.details.name)))
            .insert_header(("Content-Type", stream_wrapper.details.mime_type.to_string()))
            .streaming(stream_wrapper.stream)
    }
}

struct ErrorResponder {
    request: HttpRequest,
    settings: Settings
}

impl ErrorResponder {
    pub fn new(settings: &Settings, request: &HttpRequest) -> Self {
        Self { settings: settings.clone(), request: request.clone() }
    }

    async fn respond(&self, err: impl Error, status: StatusCode) -> HttpResponse {
        if Responder::accept_html(&self.request) {
            self.html(status).await
        } else {
            ErrorResponder::json(err, status)
        }
    }

    async fn html(&self, status: StatusCode) -> HttpResponse {
        let template = TemplateRenderer::new(&self.settings, &self.request);
        let html = match status {
            StatusCode::NOT_FOUND => template.render_localized("download_not_found.html"),
            _ => template.render_localized("error.html")
        };

        HttpResponse::build(status).body(html.await)
    }

    fn json(err: impl Error, status: StatusCode) -> HttpResponse {
        let mut response = HttpResponse::build(status);
        response.json(ErrorResponse::from(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use crate::database::setup_database_connection;
    use crate::file::error::file_read_codes;
    use crate::routes::utils::tests::{create_test_chunked_file_upload, create_test_file_upload, header_value};
    use actix_web::http::{Method, StatusCode};
    use actix_web::{test, App};
    use serial_test::serial;
    use crate::routes::error::code;

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
        assert_eq!(response_body.code, code::INVALID_PATH_PARAMETER_CODE);
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_download_200_html_ok() {
        let mut settings = Settings::load(None);
        settings.debug.enable = false;

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
