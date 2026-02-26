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

use leptos::prelude::*;
use super::header::Header;
use super::footer::Footer;

#[component]
pub fn Page(
    #[prop(into)] css_path: String,
    #[prop(into)] favicon_path: String,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <html lang="en">
            <head>
                <meta charset="UTF-8" />
                <link rel="icon" type="image/x-icon" href=favicon_path />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>"Miku Push!"</title>
                <link rel="stylesheet" href=css_path />
            </head>
            <body class="flex flex-col h-dvh bg-background text-foreground">
                <Header />
                <main class="flex flex-col flex-1">
                    {children.map(|c| c())}
                </main>
                <Footer />
            </body>
        </html>
    }
}
