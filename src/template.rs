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

use crate::config::Settings;
use accept_language::intersection;
use actix_web::HttpRequest;
use tracing::{debug, warn};

const LANGUAGE_COOKIE: &str = "language";
const DEFAULT_LANGUAGE: &str = "en";
const AVAILABLE_LANGUAGES: &[&str] = &["en", "es", "en-US", "es-ES"];

pub struct TemplateRenderer {
    settings: Settings,
    language: String,
    head: Vec<String>
}

impl TemplateRenderer {
    pub fn new(settings: &Settings, request: &HttpRequest) -> Self {
        let language = extract_language_from_cookie(request)
            .unwrap_or(extract_language_from_header(request));

        Self {
            settings: settings.clone(),
            language,
            head: Vec::new()
        }
    }

    pub fn add_to_head(&mut self, html: String) {
        self.head.push(html);
    }

    pub async fn render(&self, template: &str) -> String {
        if self.settings.debug.enable {
            self.render_from_dev_server(template).await
        } else {
            self.render_from_file(template).await
        }
    }

    pub async fn render_localized(&self, template: &str) -> String {
        self.render(format!("{}/{}", self.language, template).as_str()).await
    }

    async fn render_from_file(&self, template: &str) -> String {
        let template_dir = self.settings.server.templates_directory.clone();
        let path = std::path::Path::new(&template_dir).join(template);

        if !path.exists() {
            warn!("template file {} does not exist", path.display());
            return "".to_string();
        }

        match tokio::fs::read_to_string(&path).await {
            Ok(content) => {
                self.inject_to_head(content)
            },
            Err(err) => {
                warn!("failed to read template file {}: {}", path.display(), err);
                "".to_string()
            }
        }
    }

    async fn render_from_dev_server(&self, template: &str) -> String {
        let base_url = self.settings.debug.astro_dev_server.clone();
        let base_url = base_url.trim_end_matches('/');
        let url = format!("{}/{}", base_url, template);
        debug!("rendering template {} from astro dev server: {}", template, url);
        let response = reqwest::get(&url).await.unwrap();

        match response.text().await {
            Ok(content) => {
                self.inject_to_head(content)
            },
            Err(err) => {
                warn!("failed to read template from astro dev server {}: {}", url, err);
                "".to_string()
            }
        }
    }

    fn inject_to_head(&self, template_content: String) -> String {
        let head_elements = self.head.join("\n");
        template_content.replace("</head>", format!("{}</head>", head_elements).as_str())
    }
}

/// extracts the language from Accept-Language header, if language specified
/// is not available, then return the default language.
fn extract_language_from_header(request: &HttpRequest) -> String {
    let accept_language = request
        .headers()
        .get("Accept-Language")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let accepted_languages = intersection(accept_language, &AVAILABLE_LANGUAGES);

    let mut detected_language: Option<String> = None;
    let language = accepted_languages.first();
    if let Some(language) = language {
        detected_language = language.split("-")
            .collect::<Vec<&str>>()
            .get(0)
            .map(|language| language.to_string());
    }

    if let Some(language) = detected_language {
        return language;
    }

    DEFAULT_LANGUAGE.to_string()
}

/// extracts the language from cookie if it exists.
fn extract_language_from_cookie(request: &HttpRequest) -> Option<String> {
    let cookie = request.cookie(LANGUAGE_COOKIE);
    if let Some(cookie) = cookie {
        return Some(String::from(cookie.value()));
    }

    None
}
