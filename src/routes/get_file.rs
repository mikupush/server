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

use crate::errors::{route_error_helpers, FileInfoError};
use crate::repository::PostgresFileUploadRepository;
use crate::routes::ErrorResponse;
use crate::services::FileInfoFinder;
use actix_web::error::Result;
use actix_web::{get, web, HttpResponse};
use tracing::debug;
use uuid::Uuid;

#[get("/api/file/{id}")]
pub async fn get_file_info(
    finder: web::Data<FileInfoFinder<PostgresFileUploadRepository>>,
    id: web::Path<String>
) -> Result<HttpResponse> {
    let Ok(id) = Uuid::try_from(id.to_string()) else {
        debug!("cant convert id to uuid: {}", id.to_string());
        return Ok(route_error_helpers::invalid_uuid("id", id.to_string()))
    };

    match finder.find(id) {
        Ok(info) => Ok(HttpResponse::Ok().json(info)),
        Err(err) => Ok(handle_get_file_info_failure(err))
    }
}

fn handle_get_file_info_failure(err: FileInfoError) -> HttpResponse {
    let mut response_builder = match err {
        FileInfoError::NotExists { .. } => HttpResponse::NotFound(),
        _ => HttpResponse::InternalServerError()
    };

    response_builder.json(ErrorResponse::from(err))
}

#[cfg(test)]
mod tests {
    use crate::config::tests::setup_test_env;
    use crate::database::tests::create_test_database_connection;
    use crate::errors::{file_delete_codes, route_error_codes};
    use crate::model::{FileInfo, FileStatus};
    use crate::routes::utils::tests::{create_test_file_upload, register_test_file};
    use crate::routes::{get_file_info, ErrorResponse};
    use crate::services::FileInfoFinder;
    use actix_web::http::{Method, StatusCode};
    use actix_web::{test, web, App};
    use serial_test::serial;
    use uuid::Uuid;

    #[actix_web::test]
    async fn test_get_file_info_200_ok() {
        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileInfoFinder::test(pool.clone())))
                .service(get_file_info)
        ).await;

        let (_, file_upload) = create_test_file_upload(pool.clone());
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{}", file_upload.id.clone()).as_str())
            .method(Method::GET)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        let response = test::read_body(response).await;
        let file_info: FileInfo = serde_json::from_slice(&response).unwrap();

        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(file_upload.name, file_info.name);
        assert_eq!(file_upload.id, file_info.id);
        assert_eq!(file_upload.size, file_info.size);
        assert_eq!(file_upload.uploaded_at, file_info.uploaded_at);
        assert_eq!(file_upload.mime_type, file_info.mime_type);
        assert_eq!(FileStatus::Uploaded, file_info.status);
    }

    #[actix_web::test]
    async fn test_get_file_info_200_ok_not_uploaded_file() {
        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileInfoFinder::test(pool.clone())))
                .service(get_file_info)
        ).await;

        let file_upload = register_test_file(pool.clone());
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{}", file_upload.id.clone()).as_str())
            .method(Method::GET)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        let response = test::read_body(response).await;
        let file_info: FileInfo = serde_json::from_slice(&response).unwrap();

        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(file_upload.name, file_info.name);
        assert_eq!(file_upload.id, file_info.id);
        assert_eq!(file_upload.size, file_info.size);
        assert_eq!(file_upload.uploaded_at, file_info.uploaded_at);
        assert_eq!(file_upload.mime_type, file_info.mime_type);
        assert_eq!(FileStatus::WaitingForUpload, file_info.status);
    }

    #[actix_web::test]
    async fn test_get_file_info_404_not_found() {
        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileInfoFinder::test(pool.clone())))
                .service(get_file_info)
        ).await;

        let id = Uuid::new_v4();
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{id}").as_str())
            .method(Method::GET)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        let response = test::read_body(response).await;
        let response: ErrorResponse = serde_json::from_slice(&response).unwrap();

        assert_eq!(status_code, StatusCode::NOT_FOUND);
        assert_eq!(response.code, file_delete_codes::NOT_EXISTS_CODE);
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_file_info_400_bad_request_invalid_id() {
        setup_test_env();

        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileInfoFinder::test(pool.clone())))
                .service(get_file_info)
        ).await;

        let id = "invalid_uuid";
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{id}").as_str())
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
