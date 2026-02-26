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

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer>
            <div class="mx-auto max-w-6xl px-6 py-10 flex flex-col items-center gap-6">
                <div>
                    <p id="copyright" class="text-sm text-center mb-2">
                        "Miku Push! 2025 — made with \u{2764}\u{FE0F}"
                    </p>
                    <p class="text-sm text-center">
                        "The mascot of Miku Push! is Hatsune Miku, a character created by Crypton Future Media, INC. \u{00A9} Crypton Future Media, INC. 2007."
                    </p>
                    <p class="text-sm text-center">
                        "This project is not affiliated with or endorsed by Crypton Future Media, INC."
                    </p>
                </div>

                <div class="hidden md:block">
                    <NavLinks />
                </div>
            </div>
        </footer>
        <script>
            "const copyright = document.querySelector('#copyright');\
             const now = new Date();\
             if (copyright) { copyright.innerHTML = copyright.textContent.replace('2025', now.getFullYear().toString()) }"
        </script>
    }
}
