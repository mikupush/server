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

use crate::errors::{route_error_helpers, FileReadError};
use crate::repository::PostgresFileUploadRepository;
use crate::routes::ErrorResponse;
use crate::services::{FileRead, FileReader};
use actix_web::{get, web, HttpResponse, Result};
use tracing::debug;
use uuid::Uuid;

#[get("/u/{id}")]
pub async fn get_download(
    file_reader: web::Data<FileReader<PostgresFileUploadRepository>>,
    id: web::Path<String>,
) -> Result<HttpResponse> {
    let Ok(id) = Uuid::try_from(id.to_string()) else {
        debug!("cant convert id to uuid: {}", id.to_string());
        return Ok(route_error_helpers::invalid_uuid("id", id.to_string()))
    };

    match file_reader.read(id).await {
        Ok(details) => Ok(handle_get_download_ok(details)),
        Err(err) => Ok(handle_get_download_error(err))
    }
}

fn handle_get_download_ok(file_read: FileRead) -> HttpResponse {
    HttpResponse::Ok()
        .content_type(file_read.mime_type.clone())
        .insert_header(("Content-Length", file_read.size.to_string()))
        .insert_header(("Content-Disposition", format!("inline; filename=\"{}\"", file_read.name)))
        .insert_header(("Content-Type", file_read.mime_type.to_string()))
        .streaming(file_read.stream)
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
    use crate::routes::utils::tests::{create_test_file_upload, header_value};
    use actix_web::http::{Method, StatusCode};
    use actix_web::{test, App};
    use serial_test::serial;

    #[actix_web::test]
    #[serial]
    async fn test_get_download_200_ok() {
        let pool = setup_database_connection(&Settings::load());
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
        let pool = setup_database_connection(&Settings::load());
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
        let pool = setup_database_connection(&Settings::load());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileReader::test(pool.clone())))
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
