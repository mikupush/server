use crate::errors::{route_error_helpers, FileReadError};
use crate::routes::ErrorResponse;
use crate::services::{FileReadStream, FileReader};
use actix_web::{get, web, HttpResponse, Result};
use log::debug;
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
    use crate::routes::delete_file;
    use crate::routes::utils::tests::create_test_file_upload;
    use actix_web::http::{Method, StatusCode};
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_get_download_200_ok() {
        let pool = create_test_database_connection();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(FileReader::test(pool.clone())))
                .service(delete_file)
        ).await;

        let (_, file_upload) = create_test_file_upload(pool.clone());
        let request = test::TestRequest::default()
            .uri(format!("/u/{}", file_upload.id.clone()).as_str())
            .method(Method::GET)
            .to_request();
        let response = test::call_service(&app, request).await;
        let content_length = response.headers().get("Content-Length").unwrap().to_str().unwrap().to_string();
        let content_disposition = response.headers().get("Content-Disposition").unwrap().to_str().unwrap().to_string();
        let content_type = response.headers().get("Content-Type").unwrap().to_str().unwrap().to_string();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(content_length, file_upload.size.to_string());
        assert_eq!(content_disposition, format!("inline; filename=\"{}\"", file_upload.name));
        assert_eq!(content_type, file_upload.mime_type);
    }
}
