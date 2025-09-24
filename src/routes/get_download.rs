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

use crate::errors::{route_error_helpers, FileReadError};
use crate::routes::ErrorResponse;
use crate::services::{FileReadStream, FileReader};
use actix_web::{get, web, HttpResponse, Result};
use tracing::debug;
use uuid::Uuid;

#[get("/u/{id}")]
pub async fn get_download(
    file_reader: web::Data<FileReader>,
    id: web::Path<String>,
) -> Result<HttpResponse> {
    let Ok(id) = Uuid::try_from(id.to_string()) else {
        debug!("cant convert id to uuid: {}", id.to_string());
        return Ok(route_error_helpers::invalid_uuid("id", id.to_string()))
    };

    match file_reader.read(id) {
        Ok(details) => Ok(handle_get_download_ok(details)),
        Err(err) => Ok(handle_get_download_error(err))
    }
}

fn handle_get_download_ok(stream: FileReadStream) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(stream.mime_type.clone())
        .insert_header(("Content-Length", stream.size.to_string()))
        .insert_header(("Content-Disposition", format!("inline; filename=\"{}\"", stream.name)))
        .insert_header(("Content-Type", stream.mime_type.to_string()))
        .streaming(stream)
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
    use crate::database::tests::create_test_database_connection;
    use crate::routes::utils::tests::{create_test_file_upload, header_value};
    use actix_web::http::{Method, StatusCode};
    use actix_web::{test, App};
    use serial_test::serial;
    use crate::config::tests::setup_test_env;
    use crate::errors::{file_read_codes, route_error_codes};

    #[actix_web::test]
    #[serial]
    async fn test_get_download_200_ok() {
        setup_test_env();

        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileReader::test(pool.clone())))
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
    async fn test_get_download_404_not_found() {
        setup_test_env();

        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileReader::test(pool.clone())))
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
        setup_test_env();

        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileReader::test(pool.clone())))
                .service(get_download)
        ).await;

        let id = "invalid_uuid";
        let request = test::TestRequest::default()
            .uri(format!("/u/{}", id.clone()).as_str())
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
