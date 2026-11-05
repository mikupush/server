use std::sync::{LazyLock};
use actix_web::{web, HttpRequest, HttpResponse};
use reqwest::Method;
use tracing::debug;
use crate::config::Settings;

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .build().unwrap()
});

pub async fn astro_dev_server_proxy(
    settings: web::Data<Settings>,
    request: HttpRequest
) -> actix_web::Result<HttpResponse> {
    if !settings.debug.enable {
        return Err(actix_web::error::ErrorNotFound(""));
    }

    let path = request.path();
    let astro_path = path.starts_with("/@")
        || path.starts_with("/web")
        || path.starts_with("/node_modules");

    if !astro_path {
        return Err(actix_web::error::ErrorNotFound(""));
    }

    debug!(
        dev_server = settings.debug.astro_dev_server,
        uri = request.uri().to_string(),
        "forwarding astro dev server request"
    );

    let url = format!("{}{}",
        settings.debug.astro_dev_server.trim_end_matches("/"),
        request.uri(),
    );

    proxy_astro_dev_server(&request, url.as_str()).await
}

async fn proxy_astro_dev_server(request: &HttpRequest, url: &str) -> actix_web::Result<HttpResponse> {
    let method = request.method().as_str();
    let method: Method = method.try_into()
        .map_err(|err| actix_web::error::ErrorBadGateway(err))?;
    let mut upstream = HTTP_CLIENT.request(method, url);

    for (name, value) in request.headers() {
        if name != actix_web::http::header::HOST {
            let header_name: reqwest::header::HeaderName = name.as_str().try_into()
                .map_err(|err| actix_web::error::ErrorBadGateway(err))?;
            let header_value: reqwest::header::HeaderValue = value.as_bytes().try_into()
                .map_err(|err| actix_web::error::ErrorBadGateway(err))?;
            upstream = upstream.header(header_name, header_value);
        }
    }

    let upstream_response = upstream
        .send()
        .await
        .map_err(actix_web::error::ErrorBadGateway)?;

    let status: actix_http::StatusCode = upstream_response.status().as_str().try_into()
        .map_err(|err| actix_web::error::ErrorBadGateway(err))?;

    let mut response = HttpResponse::build(status);

    for (name, value) in upstream_response.headers() {
        let header_name: actix_http::header::HeaderName = name.as_str().try_into()
            .map_err(|err| actix_web::error::ErrorBadGateway(err))?;
        let header_value: actix_http::header::HeaderValue = value.as_bytes().try_into()
            .map_err(|err| actix_web::error::ErrorBadGateway(err))?;
        response.append_header((header_name, header_value));
    }

    let body = upstream_response
        .bytes()
        .await
        .map_err(actix_web::error::ErrorBadGateway)?;

    Ok(response.body(body))
}
