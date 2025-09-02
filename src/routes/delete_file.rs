use crate::errors::{route_error_helpers, FileDeleteError};
use crate::routes::ErrorResponse;
use crate::services::FileDeleter;
use actix_web::error::Result;
use actix_web::{delete, web, HttpResponse};
use log::debug;
use uuid::Uuid;

#[delete("/api/file/{id}")]
pub async fn delete_file(
    deleter: web::Data<FileDeleter>,
    id: web::Path<String>
) -> Result<HttpResponse> {
    let Ok(id) = Uuid::try_from(id.to_string()) else {
        debug!("cant convert id to uuid: {}", id.to_string());
        return Ok(route_error_helpers::invalid_uuid("id", id.to_string()))
    };

    match deleter.delete(id) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Ok(handle_delete_file_failure(err))
    }
}

fn handle_delete_file_failure(err: FileDeleteError) -> HttpResponse {
    let mut response_builder = match err {
        FileDeleteError::NotExists { .. } => HttpResponse::NotFound(),
        _ => HttpResponse::InternalServerError()
    };

    response_builder.json(ErrorResponse::from(err))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use actix_web::{test, web, App};
    use actix_web::http::{Method, StatusCode};
    use chrono::Utc;
    use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
    use uuid::Uuid;
    use crate::config::Upload;
    use crate::database::DbPool;
    use crate::database::tests::create_test_database_connection;
    use crate::errors::file_delete_codes;
    use crate::model::FileUpload;
    use crate::routes::{delete_file, ErrorResponse};
    use crate::schema::file_uploads;
    use crate::services::FileDeleter;

    #[actix_web::test]
    async fn test_delete_file_200_ok() {
        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileDeleter::test(pool.clone())))
                .service(delete_file)
        ).await;

        let (path, id) = create_test_file_upload(pool.clone());
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{id}").as_str())
            .method(Method::DELETE)
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert!(!path.exists(), "file should be deleted");
        assert_file_upload_deleted_in_database(id, pool.clone());
    }

    #[actix_web::test]
    async fn test_delete_file_404_not_found() {
        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileDeleter::test(pool.clone())))
                .service(delete_file)
        ).await;

        let id = Uuid::new_v4();
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{id}").as_str())
            .method(Method::DELETE)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        let response = test::read_body(response).await;
        let response: ErrorResponse = serde_json::from_slice(&response).unwrap();

        assert_eq!(status_code, StatusCode::NOT_FOUND);
        assert_eq!(response.code, file_delete_codes::NOT_EXISTS_CODE);
    }

    fn assert_file_upload_deleted_in_database(id: Uuid, pool: DbPool) {
        let mut connection = pool.get().unwrap();
        let file_upload: Option<FileUpload> = file_uploads::table
            .find(id)
            .first(&mut connection)
            .optional()
            .unwrap();

        assert!(file_upload.is_none(), "file upload should be deleted from database");
    }

    fn create_test_file_upload(pool: DbPool) -> (PathBuf, Uuid) {
        let file_upload = FileUpload {
            id: Uuid::new_v4(),
            name: format!("hatsune_miku_{}.jpg", Utc::now().timestamp()),
            mime_type: "image/jpeg".to_string(),
            size: 200792,
            uploaded_at: Utc::now().naive_utc()
        };

        let settings = Upload::test_default();
        let path = Path::new(&settings.directory())
            .join(file_upload.name.clone());
        std::fs::write(path.clone(), vec![1; file_upload.size as usize]).unwrap();

        let mut connection = pool.get().unwrap();
        diesel::insert_into(file_uploads::table)
            .values(&file_upload)
            .execute(&mut connection)
            .unwrap();

        (path, file_upload.id)
    }
}
