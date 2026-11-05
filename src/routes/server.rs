use std::io::Read;
use std::sync::OnceLock;
use actix_files::NamedFile;
use crate::config::Settings;
use crate::{SERVER_VERSION, SERVER_VERSION_CODE};
use actix_web::{get, web, HttpResponse, Responder};
use actix_web::http::StatusCode;
use log::warn;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    name: String,
    code: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerInfo {
    name: String,
    version: Version
}

#[get("/api/info")]
async fn get_server_info(settings: web::Data<Settings>) -> impl Responder {
    let info = ServerInfo {
        name: settings.server.name.clone(),
        version: Version {
            name: SERVER_VERSION.to_string(),
            code: SERVER_VERSION_CODE
        }
    };

    HttpResponse::Ok().json(info)
}

static CACHED_ICON_MIME_TYPE: OnceLock<String> = OnceLock::new();
static CACHED_ICON_CONTENT: OnceLock<Vec<u8>> = OnceLock::new();
const MAX_ICON_SIZE: usize = 1024 * 1024; // 1MB

#[get("/api/icon")]
async fn get_server_icon(settings: web::Data<Settings>) -> impl Responder {
    #[cfg(not(test))]
    if CACHED_ICON_CONTENT.get().is_some() && CACHED_ICON_MIME_TYPE.get().is_some() {
        debug!("using cached server icon");
        let content = CACHED_ICON_CONTENT.get().unwrap();
        let content_type = CACHED_ICON_MIME_TYPE.get().unwrap();

        return HttpResponse::build(StatusCode::OK)
            .content_type(content_type.clone())
            .body(content.clone())
    }

    let server = settings.server.clone();

    if server.icon.is_none() {
        return HttpResponse::NotFound().finish();
    }

    let icon_path = server.icon.unwrap();
    let icon = NamedFile::open_async(&icon_path).await;

    if let Err(err) = &icon {
        error!("error opening server icon ({}): {}", icon_path, err);
        return HttpResponse::InternalServerError().finish();
    }

    let icon = icon.unwrap();
    let content_type = icon.content_type().to_string();
    if !content_type.starts_with("image/") {
        error!("server icon is not an image ({})", icon_path);
        return HttpResponse::NotFound().finish();
    }

    let mut file = icon.file();
    let mut buffer = vec![0u8; MAX_ICON_SIZE + 1];
    let read = match file.read(&mut buffer[..]) {
        Ok(read) => read,
        Err(err) => {
            error!("error reading server icon ({}): {}", icon_path, err);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if read > MAX_ICON_SIZE {
        warn!("server icon exceeds max size ({}): 1MB", icon_path);
        return HttpResponse::NotFound().finish();
    }

    buffer.truncate(read);

    #[cfg(not(test))]
    {
        let _ = CACHED_ICON_MIME_TYPE.set(content_type.clone());
        let _ = CACHED_ICON_CONTENT.set(buffer.clone());
    }

    HttpResponse::build(StatusCode::OK)
        .content_type(content_type)
        .body(buffer)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use actix_http::{Method, StatusCode};
    use super::*;
    use actix_web::{test, App};
    use crate::routes::utils::tests::header_value;

    #[actix_web::test]
    async fn get_server_info_200() {
        let settings = Settings::load(None);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings.clone()))
                .service(get_server_info)
        ).await;

        let request = test::TestRequest::default()
            .uri("/api/info")
            .method(Method::GET)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = test::read_body(response).await;
        let body = serde_json::from_slice::<ServerInfo>(&body).unwrap();
        assert_eq!(body.name, settings.server.name);
        assert_eq!(body.version.name, SERVER_VERSION);
        assert_eq!(body.version.code, SERVER_VERSION_CODE);
    }

    #[actix_web::test]
    async fn get_server_icon_404() {
        let settings = Settings::default();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings.clone()))
                .service(get_server_icon)
        ).await;

        let request = test::TestRequest::default()
            .uri("/api/icon")
            .method(Method::GET)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn get_server_icon_404_max_size() {
        let mut settings = Settings::load(None);
        settings.server.icon = Some("examples/server_icon_large.png".to_string());

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings.clone()))
                .service(get_server_icon)
        ).await;

        let request = test::TestRequest::default()
            .uri("/api/icon")
            .method(Method::GET)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn get_server_icon_404_mime_type() {
        let mut settings = Settings::load(None);
        settings.server.icon = Some("examples/server_icon.txt".to_string());

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings.clone()))
                .service(get_server_icon)
        ).await;

        let request = test::TestRequest::default()
            .uri("/api/icon")
            .method(Method::GET)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn get_server_icon_200() {
        let mut settings = Settings::load(None);
        settings.server.icon = Some("examples/server_icon.svg".to_string());

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(settings.clone()))
                .service(get_server_icon)
        ).await;

        let request = test::TestRequest::default()
            .uri("/api/icon")
            .method(Method::GET)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);

        let content_type = header_value("Content-Type", &response);
        assert_eq!("image/svg+xml", content_type.as_str());

        let expected = fs::read("examples/server_icon.svg").unwrap();
        let body = test::read_body(response).await;
        assert_eq!(expected, body.iter().as_slice());
    }
}
