// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! View-switcher nav component (Search / Curation / Structure editor).
//!
//! Reads `AppStateContext` for the current view and `use_locale()` for labels —
//! zero props required.

use crate::app::view::AppView;
use crate::hooks::use_locale;
use crate::i18n::{
    view_label_curation_explorer, view_label_draw, view_label_explorer, view_switch_aria,
};
use crate::state::{use_app_selector, use_app_state_context};
use dioxus::prelude::*;
use ui::prelude::*;

/// Three-button view switcher.
///
/// Zero props — reads and mutates `AppStateContext` directly.
#[component]
pub fn ViewSwitch() -> Element {
    let ctx = use_app_state_context();
    let locale = use_locale();
    let mut app_state = ctx.state;
    let current_view = *use_app_selector(app_state, |state| state.view).read();

    rsx! {
        nav { aria_label: "{view_switch_aria(locale)}", style: nav_style(),
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
            r#type: "button",
            aria_pressed: if active { "true" } else { "false" },
            aria_current: if active { "page" } else { "false" },
            style: button_pill_style(active),
            onclick: move |_| on_select.call(target),
            "{label}"
        }
    }
}

fn nav_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .gap("8px")
        .property("flex-wrap", "wrap")
        .build()
}

fn button_pill_style(active: bool) -> String {
    let mut style = StyleBuilder::new()
        .property("min-width", "40px")
        .padding("3px 8px")
        .font_size("var(--fs-0)");
    if active {
        style = style
            .background_color("var(--btn-primary-bg)")
            .border("1px solid var(--btn-primary-bg)")
            .color("#fff");
    } else {
        style = style
            .color("var(--text2)")
            .background_color("color-mix(in srgb, var(--panel-bg-soft) 84%, var(--surface))")
            .border("1px solid var(--panel-border)");
    }
    style.build()
}
