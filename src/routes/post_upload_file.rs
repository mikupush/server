use crate::errors::{Error, FileUploadError, RouteError};
use crate::routes::error_response::ErrorResponse;
use crate::services::FileUploader;
use actix_web::web::Payload;
use actix_web::{post, web, HttpResponse, Result};
use log::debug;
use uuid::Uuid;
use crate::errors::route_error_helpers;

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
    use crate::database::tests::create_test_database_connection;
    use crate::database::DbPool;
    use crate::model::FileUpload;
    use crate::schema::file_uploads;
    use actix_web::http::{Method, StatusCode};
    use actix_web::{http::header::ContentType, test, App};
    use chrono::Utc;
    use diesel::RunQueryDsl;
    use crate::errors::{file_upload_codes, route_error_codes};

    #[actix_web::test]
    async fn test_post_file_200_ok() {
        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileUploader::unlimited(pool.clone())))
                .service(post_upload_file)
        ).await;

        let id = register_test_file(pool.clone());
        let file_content = std::fs::read("resources/hatsune_miku.jpg").unwrap();
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{id}/upload").as_str())
            .method(Method::POST)
            .insert_header(ContentType::octet_stream())
            .set_payload(file_content)
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_post_file_400_bad_request_invalid_id() {
        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileUploader::unlimited(pool.clone())))
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
    async fn test_post_file_400_bad_request_incomplete_file() {
        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileUploader::unlimited(pool.clone())))
                .service(post_upload_file)
        ).await;

        let id = register_test_file(pool.clone());
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
        assert_eq!(response.code, file_upload_codes::NOT_COMPLETED_CODE);
    }

    #[actix_web::test]
    async fn test_post_file_413_payload_too_large() {
        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileUploader::limited(pool.clone(), 200)))
                .service(post_upload_file)
        ).await;

        let id = register_test_file(pool.clone());
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

        assert_eq!(status_code, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(response.code, file_upload_codes::MAX_FILE_SIZE_EXCEEDED_CODE);
    }

    #[actix_web::test]
    async fn test_post_file_404_not_found() {
        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileUploader::limited(pool.clone(), 200)))
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

    fn register_test_file(pool: DbPool) -> Uuid {
        let file_upload = FileUpload {
            id: Uuid::new_v4(),
            name: format!("hatsune_miku_{}.jpg", Utc::now().timestamp()),
            mime_type: "image/jpeg".to_string(),
            size: 200792,
            uploaded_at: Utc::now().naive_utc()
        };

        let mut connection = pool.get().unwrap();
        diesel::insert_into(file_uploads::table)
            .values(&file_upload)
            .execute(&mut connection)
            .unwrap();

        file_upload.id
    }

}
