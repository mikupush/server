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
use super::nav_links::NavLinks;
use super::button::IconButton;
use super::icons::MenuIcon;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="sticky top-0 z-50 w-full">
            <div class="mx-auto flex max-w-6xl items-center justify-between px-6 py-4">
                <a href="/" class="flex items-center gap-2">
                    <img src="/assets/logo.svg" alt="Miku Push!" class="h-15" />
                    <span class="sr-only">"Go to Home"</span>
                </a>

                <nav class="hidden md:flex items-center gap-3">
                    <NavLinks />
                </nav>

                <div class="md:hidden">
                    <IconButton aria_label="Open Menu">
                        <MenuIcon class="size-6" />
                    </IconButton>
                </div>
            </div>
        </header>
    }
}
