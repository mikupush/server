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

use crate::errors::route_error_helpers;
use crate::errors::FileUploadError;
use crate::routes::error::ErrorResponse;
use crate::services::FileUploader;
use actix_web::web::Payload;
use actix_web::{post, web, HttpResponse, Result};
use tracing::debug;
use uuid::Uuid;

#[post("/api/file/{id}/upload")]
pub async fn post_upload_file(
    file_uploader: web::Data<FileUploader>,
    id: web::Path<String>,
    payload: Payload
) -> Result<HttpResponse> {
    let Ok(id) = Uuid::try_from(id.to_string()) else {
        debug!("cant convert id to uuid: {}", id.to_string());
        return Ok(route_error_helpers::invalid_uuid("id", id.to_string()))
    };

    match file_uploader.upload_file(id, payload).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Ok(handle_post_upload_file_error(err))
    }
}

fn handle_post_upload_file_error(err: FileUploadError) -> HttpResponse {
    let mut response_builder = match err {
        FileUploadError::NotCompleted => HttpResponse::BadRequest(),
        FileUploadError::MaxFileSizeExceeded => HttpResponse::PayloadTooLarge(),
        FileUploadError::NotExists { .. } => HttpResponse::NotFound(),
        _ => HttpResponse::InternalServerError()
    };

    response_builder.json(ErrorResponse::from(err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::tests::{set_test_env, setup_test_env};
    use crate::database::tests::create_test_database_connection;
    use crate::errors::{file_upload_codes, route_error_codes};
    use crate::routes::utils::tests::register_test_file;
    use actix_web::http::{Method, StatusCode};
    use actix_web::{http::header::ContentType, test, App};
    use serial_test::serial;

    #[actix_web::test]
    #[serial]
    async fn test_post_file_200_ok() {
        setup_test_env();

        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileUploader::test(pool.clone())))
                .service(post_upload_file)
        ).await;

        let file_upload = register_test_file(pool.clone());
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
        setup_test_env();

        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileUploader::test(pool.clone())))
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
        let response = test::read_body(response).await;
        let response: ErrorResponse = serde_json::from_slice(&response).unwrap();

        assert_eq!(status_code, StatusCode::BAD_REQUEST);
        assert_eq!(response.code, route_error_codes::INVALID_PATH_PARAMETER_CODE);
    }

    #[actix_web::test]
    #[serial]
    async fn test_post_file_400_bad_request_incomplete_file() {
        setup_test_env();

        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileUploader::test(pool.clone())))
                .service(post_upload_file)
        ).await;

        let file_upload = register_test_file(pool.clone());
        let bytes = vec![1u8; 100];
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{}/upload", file_upload.id).as_str())
            .method(Method::POST)
            .insert_header(ContentType::octet_stream())
            .set_payload(bytes)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        let response = test::read_body(response).await;
        let response: ErrorResponse = serde_json::from_slice(&response).unwrap();

        assert_eq!(status_code, StatusCode::BAD_REQUEST);
        assert_eq!(response.code, file_upload_codes::NOT_COMPLETED_CODE);
    }

    #[actix_web::test]
    #[serial]
    async fn test_post_file_413_payload_too_large() {
        setup_test_env();
        set_test_env("MIKU_PUSH_UPLOAD_MAX_SIZE", "200");

        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileUploader::test(pool.clone())))
                .service(post_upload_file)
        ).await;

        let file_upload = register_test_file(pool.clone());
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
        setup_test_env();

        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileUploader::test(pool.clone())))
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
        let response = test::read_body(response).await;
        let response: ErrorResponse = serde_json::from_slice(&response).unwrap();

        assert_eq!(status_code, StatusCode::NOT_FOUND);
        assert_eq!(response.code, file_upload_codes::NOT_EXISTS_CODE);
    }
}