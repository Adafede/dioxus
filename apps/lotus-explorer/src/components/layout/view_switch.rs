// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! View-switcher nav component (Explore / Curation Explorer / Draw).
//!
//! Reads `AppStateContext` for the current view and `use_locale()` for labels —
//! zero props required.

use crate::app::view::AppView;
use crate::hooks::use_locale;
use crate::i18n::{
    view_label_curation_explorer, view_label_draw, view_label_explorer, view_switch_aria,
};
use crate::state::use_app_state_context;
use dioxus::prelude::*;

/// Three-button view switcher.
///
/// Zero props — reads and mutates `AppStateContext` directly.
#[component]
pub fn ViewSwitch() -> Element {
    let ctx = use_app_state_context();
    let locale = use_locale();
    let mut app_state = ctx.state;
    let current_view = app_state.read().view;

    rsx! {
        nav {
            class: "view-switch",
            aria_label: "{view_switch_aria(locale)}",
            ViewBtn {
                label: view_label_explorer(locale),
                target: AppView::Explore,
                current: current_view,
                on_select: move |v| app_state.with_mut(|s| s.view = v),
            }
            ViewBtn {
                label: view_label_curation_explorer(locale),
                target: AppView::Curation,
                current: current_view,
                on_select: move |v| app_state.with_mut(|s| s.view = v),
            }
            ViewBtn {
                label: view_label_draw(locale),
                target: AppView::Draw,
                current: current_view,
                on_select: move |v| app_state.with_mut(|s| s.view = v),
            }
        }
    }
}

/// Single view-switch button.
#[component]
fn ViewBtn(
    label: &'static str,
    target: AppView,
    current: AppView,
    on_select: EventHandler<AppView>,
) -> Element {
    let active = current == target;
    rsx! {
        button {
            class: if active { "btn btn-xs lang-btn active" } else { "btn btn-xs lang-btn" },
            r#type: "button",
            aria_pressed: if active { "true" } else { "false" },
            onclick: move |_| on_select.call(target),
            "{label}"
        }
    }
}
