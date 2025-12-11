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
use crate::errors::route_error_helpers;
use crate::repository::PostgresFileUploadRepository;
use crate::routes::error::ErrorResponse;
use crate::services::{FileUploadError, FileUploader, CONTENT_PART_SIZE_LIMIT};
use actix_web::web::Payload;
use actix_web::{post, web, HttpRequest, HttpResponse, Result};
use futures::TryStreamExt;
use tokio_util::io::StreamReader;
use tracing::debug;
use uuid::Uuid;

#[post("/api/file/{id}/upload")]
pub async fn post_upload_file(
    settings: web::Data<Settings>,
    id: web::Path<String>,
    payload: Payload
) -> Result<HttpResponse> {
    let settings = settings.get_ref().clone();
    let file_uploader = FileUploader::get_with_settings(settings);
    let Ok(id) = Uuid::try_from(id.to_string()) else {
        debug!("cant convert id to uuid: {}", id.to_string());
        return Ok(route_error_helpers::invalid_uuid("id", id.to_string()))
    };

    let mapper = payload.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
    let body_reader = StreamReader::new(mapper);
    match file_uploader.upload_file(id, body_reader).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Ok(handle_post_upload_file_error(err))
    }
}

#[post("/api/file/{id}/upload/part/{index}")]
pub async fn post_upload_part(
    settings: web::Data<Settings>,
    path: web::Path<(String, i64)>,
    payload: Payload,
    request: HttpRequest
) -> Result<HttpResponse> {
    let (id, index) = path.into_inner();
    let settings = settings.get_ref().clone();
    let content_length = request.headers()
        .get("Content-Length")
        .and_then(|cl| cl.to_str().ok())
        .map(|value| value.parse::<u64>().ok())
        .map(|value| value.unwrap_or(0))
        .unwrap_or(0);

    if content_length > CONTENT_PART_SIZE_LIMIT {
        debug!("content length {} is greater than max part size {}", content_length, CONTENT_PART_SIZE_LIMIT);
        let response = ErrorResponse::from(FileUploadError::MaxFilePartSizeExceeded);
        return Ok(HttpResponse::PayloadTooLarge().json(response))
    }

    let file_uploader = FileUploader::get_with_settings(settings);
    let Ok(id) = Uuid::try_from(id.to_string()) else {
        debug!("cant convert id to uuid: {}", id.to_string());
        return Ok(route_error_helpers::invalid_uuid("id", id.to_string()))
    };

    let mapper = payload.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
    let body_reader = StreamReader::new(mapper);
    match file_uploader.upload_chunk(id, index, body_reader).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Ok(handle_post_upload_file_error(err))
    }
}

fn handle_post_upload_file_error(err: FileUploadError) -> HttpResponse {
    let mut response_builder = match err {
        FileUploadError::MaxFileSizeExceeded => HttpResponse::PayloadTooLarge(),
        FileUploadError::NotExists { .. } => HttpResponse::NotFound(),
        _ => HttpResponse::InternalServerError()
    };

    response_builder.json(ErrorResponse::from(err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Settings, Upload};
    use crate::database::setup_database_connection;
    use crate::errors::route_error_codes;
    use crate::routes::utils::tests::register_test_file;
    use crate::services::file_upload_codes;
    use actix_web::http::{Method, StatusCode};
    use actix_web::{http::header::ContentType, test, App};
    use serial_test::serial;
    use std::io::Read;
    use bytes::Bytes;

    #[actix_web::test]
    #[serial]
    async fn test_post_file_200_ok() {
        let settings = create_settings();
        let pool = setup_database_connection(&settings);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(post_upload_file)
        ).await;

        let file_upload = register_test_file(pool);
        let file_content = std::fs::read("resources/hatsune_miku.jpg").unwrap();
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{}/upload", file_upload.id).as_str())
            .method(Method::POST)
            .insert_header(ContentType::octet_stream())
            .set_payload(file_content)
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[actix_web::test]
    #[serial]
    async fn test_post_file_400_bad_request_invalid_id() {
        let settings = create_settings();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(post_upload_file)
        ).await;

        let id = "1"; // invalid uuid
        let bytes = vec![1u8; 100];
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{id}/upload").as_str())
            .method(Method::POST)
            .insert_header(ContentType::octet_stream())
            .set_payload(bytes)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        assert_eq!(status_code, StatusCode::BAD_REQUEST);

        let response = test::read_body(response).await;
        let response: ErrorResponse = serde_json::from_slice(&response).unwrap();
        assert_eq!(response.code, route_error_codes::INVALID_PATH_PARAMETER_CODE);
    }

    #[actix_web::test]
    #[serial]
    async fn test_post_file_413_payload_too_large() {
        let settings = create_settings_limited();
        let pool = setup_database_connection(&settings);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(post_upload_file)
        ).await;

        let file_upload = register_test_file(pool);
        let file_content = std::fs::read("resources/hatsune_miku.jpg").unwrap();
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{}/upload", file_upload.id).as_str())
            .method(Method::POST)
            .insert_header(ContentType::octet_stream())
            .set_payload(file_content)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        let response = test::read_body(response).await;
        assert_eq!(status_code, StatusCode::PAYLOAD_TOO_LARGE);

        let response: ErrorResponse = serde_json::from_slice(&response).unwrap();
        assert_eq!(response.code, file_upload_codes::MAX_FILE_SIZE_EXCEEDED_CODE);
    }

    #[actix_web::test]
    #[serial]
    async fn test_post_file_404_not_found() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(create_settings()))
                .service(post_upload_file)
        ).await;

        let id = Uuid::new_v4();
        let file_content = std::fs::read("resources/hatsune_miku.jpg").unwrap();
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{id}/upload").as_str())
            .method(Method::POST)
            .insert_header(ContentType::octet_stream())
            .set_payload(file_content)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        assert_eq!(status_code, StatusCode::NOT_FOUND);

        let response = test::read_body(response).await;
        let response: ErrorResponse = serde_json::from_slice(&response).unwrap();
        assert_eq!(response.code, file_upload_codes::NOT_EXISTS_CODE);
    }

    #[actix_web::test]
    #[serial]
    async fn test_post_file_part_200_ok() {
        let settings = create_settings();
        let pool = setup_database_connection(&settings);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(post_upload_part)
        ).await;

        let file_upload = register_test_file(pool);
        let file_content = std::fs::read("resources/hatsune_miku.jpg").unwrap();

        let request = test::TestRequest::default()
            .uri(format!("/api/file/{}/upload/part/0", file_upload.id).as_str())
            .method(Method::POST)
            .insert_header(ContentType::octet_stream())
            .set_payload(file_content)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[actix_web::test]
    #[serial]
    async fn test_post_file_part_400_bad_request_invalid_id() {
        let settings = create_settings();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(post_upload_part)
        ).await;

        let id = "1"; // invalid uuid
        let bytes = vec![1u8; 100];
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{id}/upload/part/0").as_str())
            .method(Method::POST)
            .insert_header(ContentType::octet_stream())
            .set_payload(bytes)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        assert_eq!(status_code, StatusCode::BAD_REQUEST);

        let response = test::read_body(response).await;
        let response: ErrorResponse = serde_json::from_slice(&response).unwrap();
        assert_eq!(response.code, route_error_codes::INVALID_PATH_PARAMETER_CODE);
    }

    #[actix_web::test]
    #[serial]
    async fn test_post_file_part_413_payload_too_large() {
        let settings = create_settings_limited();
        let pool = setup_database_connection(&settings);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(post_upload_part)
        ).await;

        let file_upload = register_test_file(pool);
        let file_content = std::fs::read("resources/hatsune_miku.jpg").unwrap();
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{}/upload/part/0", file_upload.id).as_str())
            .method(Method::POST)
            .insert_header(ContentType::octet_stream())
            .set_payload(file_content)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        let response = test::read_body(response).await;
        assert_eq!(status_code, StatusCode::PAYLOAD_TOO_LARGE);

        let response: ErrorResponse = serde_json::from_slice(&response).unwrap();
        assert_eq!(response.code, file_upload_codes::MAX_FILE_SIZE_EXCEEDED_CODE);
    }

    #[actix_web::test]
    #[serial]
    async fn test_post_file_part_413_payload_too_large_part() {
        let settings = create_settings();
        let pool = setup_database_connection(&settings);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings))
                .service(post_upload_part)
        ).await;

        let file_upload = register_test_file(pool);
        let file_content = Bytes::from(vec![1u8; (CONTENT_PART_SIZE_LIMIT + 1) as usize]);
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{}/upload/part/0", file_upload.id).as_str())
            .method(Method::POST)
            .insert_header(ContentType::octet_stream())
            .set_payload(file_content)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        let response = test::read_body(response).await;
        assert_eq!(status_code, StatusCode::PAYLOAD_TOO_LARGE);

        let response: ErrorResponse = serde_json::from_slice(&response).unwrap();
        assert_eq!(response.code, file_upload_codes::MAX_FILE_PART_SIZE_EXCEEDED_CODE);
    }

    #[actix_web::test]
    #[serial]
    async fn test_post_file_part_404_not_found() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(create_settings()))
                .service(post_upload_part)
        ).await;

        let id = Uuid::new_v4();
        let file_content = std::fs::read("resources/hatsune_miku.jpg").unwrap();
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{id}/upload/part/0").as_str())
            .method(Method::POST)
            .insert_header(ContentType::octet_stream())
            .set_payload(file_content)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        assert_eq!(status_code, StatusCode::NOT_FOUND);

        let response = test::read_body(response).await;
        let response: ErrorResponse = serde_json::from_slice(&response).unwrap();
        assert_eq!(response.code, file_upload_codes::NOT_EXISTS_CODE);
    }

    fn create_settings() -> Settings {
        let mut settings = Settings::load();
        settings.upload = Upload::create();
        settings
    }

    fn create_settings_limited() -> Settings {
        let mut settings = Settings::load();
        settings.upload = Upload::create_with_limit(200);
        settings
    }
}
