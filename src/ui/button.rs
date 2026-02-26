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

const BASE_CLASSES: &str = "focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:aria-invalid:border-destructive/50 rounded-lg border border-transparent bg-clip-padding text-sm font-medium focus-visible:ring-3 aria-invalid:ring-3 [&_svg:not([class*='size-'])]:size-4 inline-flex items-center justify-center whitespace-nowrap transition-all disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none shrink-0 [&_svg]:shrink-0 outline-none group/button select-none";

#[allow(dead_code)]
pub enum ButtonVariant {
    Default,
    Ghost,
}

#[allow(dead_code)]
pub enum ButtonSize {
    Default,
    Lg,
    IconLg,
}

impl ButtonVariant {
    fn classes(&self) -> &'static str {
        match self {
            ButtonVariant::Default => "bg-primary text-primary-foreground hover:bg-primary/80",
            ButtonVariant::Ghost => "hover:bg-muted hover:text-foreground dark:hover:bg-muted/50 aria-expanded:bg-muted aria-expanded:text-foreground",
        }
    }
}

impl ButtonSize {
    fn classes(&self) -> &'static str {
        match self {
            ButtonSize::Default => "h-8 gap-1.5 px-2.5 has-data-[icon=inline-end]:pr-2 has-data-[icon=inline-start]:pl-2",
            ButtonSize::Lg => "h-9 gap-1.5 px-2.5 has-data-[icon=inline-end]:pr-3 has-data-[icon=inline-start]:pl-3",
            ButtonSize::IconLg => "size-9",
        }
    }
}

pub fn button_classes(variant: ButtonVariant, size: ButtonSize, extra: &str) -> String {
    format!("{} {} {} {}", BASE_CLASSES, variant.classes(), size.classes(), extra)
}

#[component]
pub fn LinkButton(
    href: &'static str,
    #[prop(optional)] target: Option<&'static str>,
    #[prop(optional)] rel: Option<&'static str>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let classes = button_classes(ButtonVariant::Ghost, ButtonSize::Lg, &class);

    view! {
        <a
            href=href
            target=target
            rel=rel
            data-slot="button"
            data-variant="ghost"
            data-size="lg"
            class=classes
        >
            {children()}
        </a>
    }
}

#[component]
pub fn IconButton(
    #[prop(optional, into)] class: String,
    #[prop(optional)] aria_label: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let classes = button_classes(ButtonVariant::Ghost, ButtonSize::IconLg, &class);

    view! {
        <button
            data-slot="button"
            data-variant="ghost"
            data-size="icon-lg"
            class=classes
            aria-label=aria_label
        >
            {children()}
        </button>
    }
}
