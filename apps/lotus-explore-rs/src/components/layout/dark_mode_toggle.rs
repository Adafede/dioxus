// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Light / dark theme toggle in the page header.

use crate::hooks::use_locale;
use crate::i18n::{TextKey, t};
use crate::state::use_app_state_context;
use dioxus::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn DarkModeToggle() -> Element {
    let locale = use_locale();
    let ctx = use_app_state_context();
    let mut app_state = ctx.state;
    let dark_mode = app_state.read().dark_mode;
    let label = if dark_mode { "Dark" } else { "Light" };

    rsx! {
        button {
            class: "theme-toggle inline-flex shrink-0 cursor-pointer items-center gap-2 rounded-full border border-border bg-surface px-3 py-1.5 text-muted shadow-xs transition-all duration-150 hover:border-accent/50 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/40",
            r#type: "button",
            role: "switch",
            "aria-label": t(locale, TextKey::DarkModeToggle),
            "aria-checked": dark_mode.to_string(),
            onclick: move |_| {
                let new_dark_mode = !dark_mode;
                app_state.with_mut(|s| s.dark_mode = new_dark_mode);

                // Persist to localStorage
                #[cfg(target_arch = "wasm32")]
                {
                    if let Some(win) = web_sys::window()
                        && let Ok(storage) = js_sys::Reflect::get(&win, &"localStorage".into())
                        && !storage.is_undefined()
                        && let Ok(func) = js_sys::Reflect::get(&storage, &"setItem".into())
                        && let Some(set_item) = func.dyn_ref::<js_sys::Function>()
                    {
                        let _ = set_item.call2(&storage, &"dark_mode".into(), &(if new_dark_mode { "true" } else { "false" }).into());
                    }
                }
            },
            span {
                class: "flex h-6 w-6 items-center justify-center rounded-full bg-accent/12 text-accent",
                if dark_mode {
                    span { class: "text-xs font-bold", "🌙" }
                } else {
                    span { class: "text-xs font-bold", "🔆" }
                }
            }
            span {
                class: "text-[11px] font-semibold uppercase tracking-[0.12em]",
                "{label}"
            }
        }
    }
}
