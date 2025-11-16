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

use crate::errors::FileUploadError;
use crate::repository::PostgresFileUploadRepository;
use crate::routes::error::ErrorResponse;
use crate::services::FileRegister;
use actix_web::{post, web, HttpResponse, Result};
use tracing::debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCreate {
    #[serde(with = "crate::serialization::uuid")]
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
}

#[post("/api/file")]
pub async fn post_file(
    request: web::Json<FileCreate>,
    file_upload_register: web::Data<FileRegister<PostgresFileUploadRepository>>
) -> Result<HttpResponse> {
    let request = request.into_inner();

    match file_upload_register.register_file(request.clone()) {
        Ok(_) => {
            debug!("returning status code 200 for registered file {}", request.id);
            Ok(HttpResponse::Ok().finish())
        },
        Err(err) => Ok(handle_register_file_failure(request, err))
    }
}

fn handle_register_file_failure(request: FileCreate, err: FileUploadError) -> HttpResponse {
    debug!("handling register file error: {}: {}", request.id, err);

    let mut response_builder = match err {
        FileUploadError::Exists => HttpResponse::Conflict(),
        FileUploadError::MaxFileSizeExceeded => HttpResponse::PayloadTooLarge(),
        _ => HttpResponse::InternalServerError()
    };

    let response = response_builder.json(ErrorResponse::from(err));
    debug!("returning error status code {} for file register with id {}", response.status(), request.id);
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::tests::{set_test_env, setup_test_env};
    use crate::errors::{file_upload_codes, route_error_codes};
    use actix_web::http::{Method, StatusCode};
    use actix_web::{http::header::ContentType, test, App};
    use serial_test::serial;
    use crate::routes::json_error_handler;

    #[actix_web::test]
    #[serial]
    async fn test_post_file_200_ok() {
        setup_test_env();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileRegister::test()))
                .service(post_file)
        ).await;
        let body = FileCreate {
            id: Uuid::new_v4(),
            name: "hatsune_miku.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            size: 200792,
        };
        let request = test::TestRequest::default()
            .uri("/api/file")
            .method(Method::POST)
            .insert_header(ContentType::json())
            .set_json(body)
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[actix_web::test]
    #[serial]
    async fn test_post_file_409_conflict() {
        setup_test_env();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileRegister::test()))
                .service(post_file)
        ).await;
        let body = FileCreate {
            id: Uuid::new_v4(),
            name: "hatsune_miku.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            size: 200792,
        };
        let request = test::TestRequest::default()
            .uri("/api/file")
            .method(Method::POST)
            .insert_header(ContentType::json())
            .set_json(body.clone())
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);

        let request = test::TestRequest::default()
            .uri("/api/file")
            .method(Method::POST)
            .insert_header(ContentType::json())
            .set_json(body.clone())
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        let response_body = test::read_body(response).await;
        let response_body = serde_json::from_slice::<ErrorResponse>(&response_body).unwrap();

        assert_eq!(status_code, StatusCode::CONFLICT);
        assert_eq!(response_body.code, file_upload_codes::EXISTS_CODE);
    }

    #[actix_web::test]
    #[serial]
    async fn test_post_file_413_payload_too_large() {
        setup_test_env();
        set_test_env("MIKU_PUSH_UPLOAD_MAX_SIZE", "200");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileRegister::test()))
                .service(post_file)
        ).await;

        let body = FileCreate {
            id: Uuid::new_v4(),
            name: "hatsune_miku.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            size: 200792,
        };
        let request = test::TestRequest::default()
            .uri("/api/file")
            .method(Method::POST)
            .insert_header(ContentType::json())
            .set_json(body.clone())
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        let response_body = test::read_body(response).await;

        assert_eq!(status_code, StatusCode::PAYLOAD_TOO_LARGE);

        let response_body = serde_json::from_slice::<ErrorResponse>(&response_body).unwrap();
        assert_eq!(response_body.code, file_upload_codes::MAX_FILE_SIZE_EXCEEDED_CODE);
    }

    #[actix_web::test]
    #[serial]
    async fn test_post_file_400_bad_request() {
        setup_test_env();

        let app = test::init_service(
            App::new()
                .app_data(web::JsonConfig::default().error_handler(json_error_handler))
                .app_data(web::Data::new(FileRegister::test()))
                .service(post_file)
        ).await;

        let body = r#"
        {
            "id": "invalid_id",
            "name": "hatsune_miku.jpg",
            "mime_type": "image/jpeg",
            "size": 200792
        }
        "#;
        let request = test::TestRequest::default()
            .uri("/api/file")
            .method(Method::POST)
            .insert_header(ContentType::json())
            .set_payload(body)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        let response_body = test::read_body(response).await;
        println!("status code: {:?}", status_code.clone());
        println!("response_body: {:?}", response_body);

        assert_eq!(status_code, StatusCode::BAD_REQUEST);

        let response_body = serde_json::from_slice::<ErrorResponse>(&response_body).unwrap();
        assert_eq!(response_body.code, route_error_codes::INVALID_REQUEST_BODY_CODE);
    }
}
