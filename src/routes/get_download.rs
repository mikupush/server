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
use crate::file::{FileInfo, FileInfoFinder, FileReader, FileUpload, SingleFileReader};
use crate::template::TemplateRenderer;
use crate::tracing::ElapsedTimeTracing;
use actix_http::StatusCode;
use actix_web::{get, mime, web, HttpRequest, HttpResponse, HttpResponseBuilder};
use actix_web::http::header::{Accept, ACCEPT, USER_AGENT};
use log::warn;
use serde_json::json;
use tracing::debug;
use uuid::Uuid;
use crate::routes::error::helper;
use crate::routes::utils::range_header;
use crate::schema::file_uploads::dsl::file_uploads;

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

    let success_responder = Responder::new(&id, &file_upload, &settings, &request);
    let response = success_responder.respond().await;
    time_tracing.trace();

    response
}

struct Responder {
    request: HttpRequest,
    settings: Settings,
    id: Uuid,
    mime_type: String,
    size: u64,
    chunked: bool,
}

impl Responder {
    pub fn new(
        id: &Uuid,
        file_info: &FileInfo,
        settings: &Settings,
        request: &HttpRequest
    ) -> Self {
        Self {
            id: id.clone(),
            mime_type: file_info.mime_type.clone(),
            chunked: file_info.chunked,
            size: file_info.size as u64,
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

    fn accept_range(&self) -> bool {
        let supported_content_type =
            self.mime_type.starts_with("video/")
            || self.mime_type.starts_with("audio/");

        !self.chunked && supported_content_type
    }

    pub async fn respond(&self) -> HttpResponse {
        let force_raw = self.request
            .query_string()
            .contains("raw");

        if force_raw || (self.is_robot_request() && self.is_crawlable()) {
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
        let is_range_request = self.request.headers()
            .contains_key("Range");

        if is_range_request && !self.accept_range() {
            return self.respond_range_not_satisfied();
        }

        if is_range_request {
            self.raw_range().await
        } else {
            self.raw_all().await
        }
    }

    async fn raw_all(&self) -> HttpResponse {
        let file_reader = FileReader::get_with_settings(&self.settings);
        let stream_wrapper = match file_reader.read(self.id).await {
            Ok(stream_wrapper) => stream_wrapper,
            Err(err) => {
                warn!("unable to respond with file contents: {}", err);
                return HttpResponse::InternalServerError().finish();
            }
        };

        let mut response = HttpResponse::Ok();
        response.content_type(stream_wrapper.details.mime_type.clone());
        response.insert_header(("Content-Disposition", format!("inline; filename=\"{}\"", stream_wrapper.details.name)));
        self.insert_raw_common_headers(&mut response);

        response.streaming(stream_wrapper.stream)
    }

    async fn raw_range(&self) -> HttpResponse {
        let (start, end) = match range_header(&self.request, self.size) {
            Some(range) => range,
            None => return HttpResponse::BadRequest().finish(),
        };

        let file_reader = FileReader::get_with_settings(&self.settings);
        let result = file_reader.read_range(self.id, start, end).await;
        if let Err(err) = &result {
            return match err {
                FileReadError::NotExists { .. } => HttpResponse::NotFound().finish(),
                FileReadError::RangeNotAllowed { .. } => self.respond_range_not_satisfied(),
                _ => HttpResponse::InternalServerError().finish(),
            };
        }

        let stream_wrapper = result.unwrap();
        let mut response = HttpResponse::PartialContent();
        response.content_type(stream_wrapper.details.mime_type.clone());
        response.insert_header(("Content-Disposition", format!("inline; filename=\"{}\"", stream_wrapper.details.name)));
        response.insert_header(("Content-Range", format!("bytes {}-{}/{}", start, end, self.size)));
        self.insert_raw_common_headers(&mut response);

        response.streaming(stream_wrapper.stream)
    }

    fn insert_raw_common_headers(&self, response: &mut HttpResponseBuilder) {
        response.insert_header(("Content-Length", self.size.to_string()));
        response.insert_header(("Content-Type", self.mime_type.to_string()));

        if self.accept_range() {
            response.insert_header(("Accept-Ranges", "bytes"));
        } else {
            response.insert_header(("Accept-Ranges", "none"));
        }
    }

    fn respond_range_not_satisfied(&self) -> HttpResponse {
        let mut response = HttpResponse::RangeNotSatisfiable();
        response.content_type(self.mime_type.clone());
        response.insert_header(("Content-Range", format!("bytes */{}", self.size)));
        self.insert_raw_common_headers(&mut response);
        response.finish()
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
    use crate::routes::utils::tests::{header_value, FileCreateFactories, IntegrationTestFileUploadFactory, TEST_FILE_CONTENT_LENGTH};
    use actix_web::http::{Method, StatusCode};
    use actix_web::{test, App};
    use serial_test::serial;
    use crate::file::FileUpload;
    use crate::routes::error::code;

    #[actix_web::test]
    #[serial]
    async fn test_get_download_200_ok() {
        let settings = Settings::load(None);
        let pool = setup_database_connection(&settings);
        let factory = IntegrationTestFileUploadFactory::new(&settings, &pool);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(get_download)
        ).await;

        let (_, file_upload) = factory.create(FileCreateFactories::text_plain()).await;
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
        let factory = IntegrationTestFileUploadFactory::new(&settings, &pool);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings.clone()))
                .service(get_download)
        ).await;

        let (content, file_create_request) = FileCreateFactories::text_plain();
        let file_upload = factory.create_chunked((content.clone(), file_create_request)).await;
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
        assert_eq!(body, content);
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
        let factory = IntegrationTestFileUploadFactory::new(&settings, &pool);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(get_download)
        ).await;

        let (_, file_upload) = factory.create(FileCreateFactories::text_plain()).await;
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
        let factory = IntegrationTestFileUploadFactory::new(&settings, &pool);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(get_download)
        ).await;

        let (_, file_upload) = factory.create(FileCreateFactories::image_png()).await;
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

    #[actix_web::test]
    #[serial]
    async fn test_get_download_200_fallback_html_when_discord() {
        let settings = Settings::load(None);
        let pool = setup_database_connection(&settings);
        let factory = IntegrationTestFileUploadFactory::new(&settings, &pool);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(get_download)
        ).await;

        let (_, file_upload) = factory.create(FileCreateFactories::text_plain()).await;
        let request = test::TestRequest::default()
            .uri(format!("/u/{}", file_upload.id.clone()).as_str())
            .method(Method::GET)
            .insert_header(("Accept", "text/html"))
            .insert_header(("User-Agent", "Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com)"))
            .to_request();
        let response = test::call_service(&app, request).await;
        let content_type = header_value("Content-Type", &response);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(content_type, "text/html; charset=utf-8");
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_download_200_raw_supporting_range() {
        let settings = Settings::load(None);
        let pool = setup_database_connection(&settings);
        let factory = IntegrationTestFileUploadFactory::new(&settings, &pool);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(get_download)
        ).await;

        let (_, file_upload) = factory.create(FileCreateFactories::video_mp4()).await;
        let request = test::TestRequest::default()
            .uri(format!("/u/{}", file_upload.id.clone()).as_str())
            .method(Method::GET)
            .to_request();
        let response = test::call_service(&app, request).await;
        let accept_range = header_value("Accept-Ranges", &response);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(accept_range, "bytes");
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_download_200_raw_not_supporting_range() {
        let settings = Settings::load(None);
        let pool = setup_database_connection(&settings);
        let factory = IntegrationTestFileUploadFactory::new(&settings, &pool);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(get_download)
        ).await;

        let (_, file_upload) = factory.create(FileCreateFactories::text_plain()).await;
        let request = test::TestRequest::default()
            .uri(format!("/u/{}", file_upload.id.clone()).as_str())
            .method(Method::GET)
            .to_request();
        let response = test::call_service(&app, request).await;
        let accept_range = header_value("Accept-Ranges", &response);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(accept_range, "none");
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_download_200_raw_chunked_not_supporting_range() {
        let settings = Settings::load(None);
        let pool = setup_database_connection(&settings);
        let factory = IntegrationTestFileUploadFactory::new(&settings, &pool);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(get_download)
        ).await;

        let file_upload = factory.create_chunked(FileCreateFactories::video_mp4()).await;
        let request = test::TestRequest::default()
            .uri(format!("/u/{}", file_upload.id.clone()).as_str())
            .method(Method::GET)
            .to_request();
        let response = test::call_service(&app, request).await;
        let accept_range = header_value("Accept-Ranges", &response);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(accept_range, "none");
    }

    async fn assert_get_download_416_raw_not_supporting_range(
        settings: &Settings,
        file_upload: &FileUpload
    ) {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings.clone()))
                .service(get_download)
        ).await;

        let initial_range = TEST_FILE_CONTENT_LENGTH / 2;
        let request = test::TestRequest::default()
            .uri(format!("/u/{}", file_upload.id.clone()).as_str())
            .method(Method::GET)
            .insert_header(("Range", format!("bytes={}-", initial_range)))
            .to_request();
        let response = test::call_service(&app, request).await;
        let content_range = header_value("Content-Range", &response);
        let content_length = header_value("Content-Length", &response);

        assert_eq!(response.status(), StatusCode::RANGE_NOT_SATISFIABLE);
        assert_eq!(content_length, file_upload.size.to_string());
        assert_eq!(content_range, format!("bytes */{}", file_upload.size));
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_download_416_raw_not_supporting_range() {
        let settings = Settings::load(None);
        let pool = setup_database_connection(&settings);
        let factory = IntegrationTestFileUploadFactory::new(&settings, &pool);

        let (_, file_upload) = factory.create(FileCreateFactories::text_plain()).await;
        assert_get_download_416_raw_not_supporting_range(&settings, &file_upload).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_download_416_raw_chunked_not_supporting_range() {
        let settings = Settings::load(None);
        let pool = setup_database_connection(&settings);
        let factory = IntegrationTestFileUploadFactory::new(&settings, &pool);

        let file_upload = factory.create_chunked(FileCreateFactories::video_mp4()).await;
        assert_get_download_416_raw_not_supporting_range(&settings, &file_upload).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_download_206_raw_partial_content() {
        let settings = Settings::load(None);
        let pool = setup_database_connection(&settings);
        let factory = IntegrationTestFileUploadFactory::new(&settings, &pool);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(get_download)
        ).await;

        let initial_range = TEST_FILE_CONTENT_LENGTH / 2;
        let (_, file_upload) = factory.create(FileCreateFactories::video_mp4()).await;
        let request = test::TestRequest::default()
            .uri(format!("/u/{}", file_upload.id.clone()).as_str())
            .method(Method::GET)
            .insert_header(("Range", format!("bytes={}-", initial_range)))
            .to_request();
        let response = test::call_service(&app, request).await;
        let content_range = header_value("Content-Range", &response);
        let content_length = header_value("Content-Length", &response);

        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(content_length, file_upload.size.to_string());
        assert_eq!(content_range, format!("bytes {}-{}/{}", initial_range, file_upload.size - 1, file_upload.size));
    }
}
