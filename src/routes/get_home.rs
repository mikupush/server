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

use actix_web::{get, web, HttpResponse};
use leptos::prelude::*;
use leptos::tachys::view::RenderHtml;
use crate::ui::Page;

#[derive(Debug, Clone)]
pub struct StaticAssets {
    pub css_path: String,
    pub favicon_path: String,
}

impl StaticAssets {
    pub fn discover(static_directory: &str, base_path: &str) -> Self {
        let dir = std::path::Path::new(static_directory);

        let css_path = find_asset(dir, base_path, ".css")
            .unwrap_or_else(|| format!("{}/style.css", base_path));

        let favicon_path = find_asset(dir, base_path, ".ico")
            .unwrap_or_else(|| format!("{}/favicon.ico", base_path));

        tracing::info!("discovered static assets - css: {}, favicon: {}", css_path, favicon_path);

        Self { css_path, favicon_path }
    }
}

fn find_asset(dir: &std::path::Path, base_path: &str, extension: &str) -> Option<String> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.ends_with(extension) {
            return Some(format!("{}/{}", base_path, name));
        }
    }
    None
}

#[get("/")]
pub async fn get_home(
    assets: web::Data<StaticAssets>,
) -> HttpResponse {
    let css_path = assets.css_path.clone();
    let favicon_path = assets.favicon_path.clone();

    let owner = Owner::new();
    let html = owner.with(|| {
        view! {
            <Page css_path=css_path favicon_path=favicon_path />
        }
        .to_html()
    });

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!("<!DOCTYPE html>{}", html))
}
