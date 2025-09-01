use actix_web::{post, web, HttpResponse, Result};
use uuid::Uuid;
use log::debug;
use serde::{Deserialize, Serialize};
use crate::errors::FileUploadError;
use crate::routes::response::ErrorResponse;
use crate::services::FileRegister;

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
    file_upload_register: web::Data<FileRegister>
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
    use actix_web::{http::header::ContentType, test, App};
    use actix_web::http::{Method, StatusCode};
    use crate::database::tests::create_test_database_connection;
    use crate::services::FileSizeLimiter;
    use super::*;

    #[actix_web::test]
    async fn test_post_file_200_ok() {
        let service = FileRegister::new(
            create_test_database_connection(),
            FileSizeLimiter::unlimited()
        );
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(service))
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
    async fn test_post_file_409_conflict() {
        let service = FileRegister::new(
            create_test_database_connection(),
            FileSizeLimiter::unlimited()
        );
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(service))
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
        assert_eq!(response_body.code, FileUploadError::Exists.code());
    }

    #[actix_web::test]
    async fn test_post_file_413_payload_too_large() {
        let service = FileRegister::new(
            create_test_database_connection(),
            FileSizeLimiter::limited(200)
        );
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(service))
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
        let response_body = serde_json::from_slice::<ErrorResponse>(&response_body).unwrap();

        assert_eq!(status_code, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(response_body.code, FileUploadError::MaxFileSizeExceeded.code());
    }
}
