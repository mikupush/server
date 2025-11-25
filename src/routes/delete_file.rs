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

use crate::errors::{route_error_helpers, FileDeleteError};
use crate::repository::PostgresFileUploadRepository;
use crate::routes::ErrorResponse;
use crate::services::FileDeleter;
use actix_web::error::Result;
use actix_web::{delete, web, HttpResponse};
use tracing::debug;
use uuid::Uuid;

#[delete("/api/file/{id}")]
pub async fn delete_file(
    deleter: web::Data<FileDeleter<PostgresFileUploadRepository>>,
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
    use crate::config::tests::setup_test_env;
    use crate::config::Settings;
    use crate::database::{setup_database_connection, DbPool};
    use crate::errors::{file_delete_codes, route_error_codes};
    use crate::model::FileUploadModel;
    use crate::routes::utils::tests::create_test_file_upload;
    use crate::routes::{delete_file, ErrorResponse};
    use crate::schema::file_uploads;
    use crate::services::FileDeleter;
    use actix_web::http::{Method, StatusCode};
    use actix_web::{test, web, App};
    use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
    use serial_test::serial;
    use uuid::Uuid;

    #[actix_web::test]
    async fn test_delete_file_200_ok() {
        let pool = setup_database_connection(&Settings::load());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileDeleter::test(pool.clone())))
                .service(delete_file)
        ).await;

        let (path, file_upload) = create_test_file_upload(pool.clone());
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{}", file_upload.id.clone()).as_str())
            .method(Method::DELETE)
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert!(!path.exists(), "file should be deleted");
        assert_file_upload_deleted_in_database(file_upload.id, pool.clone());
    }

    #[actix_web::test]
    async fn test_delete_file_404_not_found() {
        let pool = setup_database_connection(&Settings::load());
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

    #[actix_web::test]
    #[serial]
    async fn test_delete_file_400_bad_request_invalid_id() {
        setup_test_env();

        let pool = setup_database_connection(&Settings::load());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileDeleter::test(pool.clone())))
                .service(delete_file)
        ).await;

        let id = "invalid_uuid";
        let request = test::TestRequest::default()
            .uri(format!("/api/file/{id}").as_str())
            .method(Method::DELETE)
            .to_request();
        let response = test::call_service(&app, request).await;
        let status_code = response.status().clone();
        let response_body = test::read_body(response).await;

        assert_eq!(status_code, StatusCode::BAD_REQUEST);

        let response_body = serde_json::from_slice::<ErrorResponse>(&response_body).unwrap();
        assert_eq!(response_body.code, route_error_codes::INVALID_PATH_PARAMETER_CODE);
    }

    fn assert_file_upload_deleted_in_database(id: Uuid, pool: DbPool) {
        let mut connection = pool.get().unwrap();
        let file_upload: Option<FileUploadModel> = file_uploads::table
            .find(id)
            .first(&mut connection)
            .optional()
            .unwrap();

        assert!(file_upload.is_none(), "file upload should be deleted from database");
    }
}
