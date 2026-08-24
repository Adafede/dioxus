// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Light / dark theme toggle button, placed in the page header next to the
//! language switcher.
//!
//! Zero props — reads and mutates `AppStateContext` for the `dark_mode` boolean.
//! The `AppRuntimeEffects` component (in `shell.rs`) syncs
//! `data-theme="dark" | "light"` on `<html>` whenever `app_state.dark_mode`
//! changes, and persists `?dark_mode=true` in the URL.

use crate::hooks::use_locale;
use crate::i18n::{TextKey, t};
use crate::state::use_app_state_context;
use dioxus::prelude::*;

/// Toggle between light and dark themes.
///
/// Reads `dark_mode` from `AppStateContext` and writes it back on click.
/// Uses proper ARIA attributes for accessibility.
/// Uses localized aria-label for internationalization.
#[component]
pub fn DarkModeToggle() -> Element {
    let locale = use_locale();
    let ctx = use_app_state_context();
    let mut app_state = ctx.state;
    let dark_mode = app_state.read().dark_mode;

    rsx! {
        button {
            class: "theme-toggle",
            r#type: "button",
            role: "switch",
            "aria-label": t(locale, TextKey::DarkModeToggle),
            "aria-checked": dark_mode.to_string(),
            onclick: move |_| {
                app_state.with_mut(|s| s.dark_mode = !s.dark_mode);
            },
            // type="button" prevents accidental form submission if this
            // component is ever placed inside a <form> element.
            { if dark_mode {
                // Moon shown when dark mode is active → click to go light.
                rsx! { span { "🌙" } }
            } else {
                // Sun shown when light mode is active → click to go dark.
                rsx! { span { "☀️" } }
            }}
        }
    }
}
