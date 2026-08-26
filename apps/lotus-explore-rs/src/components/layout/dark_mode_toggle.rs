// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Light / dark theme toggle in the page header.

use crate::hooks::use_locale;
use crate::i18n::{TextKey, t};
use crate::state::use_app_state_context;
use dioxus::prelude::*;

#[component]
pub fn DarkModeToggle() -> Element {
    let locale = use_locale();
    let ctx = use_app_state_context();
    let mut app_state = ctx.state;
    let dark_mode = app_state.read().dark_mode;
    let label = if dark_mode { "Dark" } else { "Light" };

    rsx! {
        button {
            class: "theme-toggle inline-flex cursor-pointer items-center gap-2 rounded-full border border-border bg-surface px-2.5 py-1.5 text-muted shadow-xs transition-colors hover:border-accent/50 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/40",
            r#type: "button",
            role: "switch",
            "aria-label": t(locale, TextKey::DarkModeToggle),
            "aria-checked": dark_mode.to_string(),
            onclick: move |_| {
                app_state.with_mut(|s| s.dark_mode = !s.dark_mode);
            },
            span {
                class: "flex size-6 items-center justify-center rounded-full bg-accent/12 text-accent",
                if dark_mode {
                    span { class: "text-sm leading-none", "🌙" }
                } else {
                    span { class: "text-sm leading-none", "☀️" }
                }
            }
            span {
                class: "text-[11px] font-semibold uppercase tracking-[0.12em]",
                "{label}"
            }
        }
    }
}
