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
use crate::repository::PostgresFileUploadRepository;
use crate::routes::ErrorResponse;
use crate::services::{FileReader, FileStreamWrapper, SingleFileReader};
use actix_web::{get, web, HttpResponse, Result};
use tracing::debug;
use uuid::Uuid;

#[get("/u/{id}")]
pub async fn get_download(
    settings: web::Data<Settings>,
    id: web::Path<String>,
) -> Result<HttpResponse> {
    let file_reader = FileReader::get_with_settings(settings.get_ref().clone());
    let Ok(id) = Uuid::try_from(id.to_string()) else {
        debug!("cant convert id to uuid: {}", id.to_string());
        return Ok(route_error_helpers::invalid_uuid("id", id.to_string()))
    };

    match file_reader.read(id).await {
        Ok(stream_wrapper) => Ok(handle_get_download_ok(stream_wrapper)),
        Err(err) => Ok(handle_get_download_error(err))
    }
}

fn handle_get_download_ok(stream_wrapper: FileStreamWrapper) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(stream_wrapper.details.mime_type.clone())
        .insert_header(("Content-Length", stream_wrapper.details.size.to_string()))
        .insert_header(("Content-Disposition", format!("inline; filename=\"{}\"", stream_wrapper.details.name)))
        .insert_header(("Content-Type", stream_wrapper.details.mime_type.to_string()))
        .streaming(stream_wrapper.stream)
}

fn handle_get_download_error(err: FileReadError) -> HttpResponse {
    let mut response_builder = match err {
        FileReadError::NotExists { .. } => HttpResponse::NotFound(),
        _ => HttpResponse::InternalServerError()
    };

    response_builder.json(ErrorResponse::from(err))
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
        let settings = Settings::load();
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
        let settings = Settings::load();
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
        let settings = Settings::load();
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
        let settings = Settings::load();
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
}
