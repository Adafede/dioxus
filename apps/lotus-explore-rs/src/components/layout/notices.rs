// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Status, warning, and error notice components.
//!
//! All notice components read locale via `use_locale()` and explore state via
//! `ResultsContext` — no `explore` or `locale` props are drilled from `App`.

use crate::components::copy_button::CopyButton;
use crate::features::explore::interactions::use_explore_interactions;
use crate::features::explore::recovery;
use crate::features::explore::selectors::{use_lifecycle_selector, use_result_selector};
use crate::features::explore::types::ErrorKind;
use crate::features::explore::url_state::absolute_share_url;
use crate::i18n::{TextKey, t};
use crate::services::error_presenter::{
    error_hint_text, format_domain_error, format_taxon_warning,
};
use crate::state::use_results_context;
use dioxus::prelude::*;
use std::sync::Arc;
use ui::prelude::*;

/// Share URL notice — shows the current shareable URL with a copy button.
#[component]
pub fn ShareNotice(shareable_url: Memo<Option<Arc<str>>>) -> Element {
    let locale = crate::hooks::use_locale();
    let share_input_id = "share-url-field";
    let share = shareable_url.read();
    let Some(share) = share.as_deref() else {
        return rsx! {};
    };
    rsx! {
        div { role: "status", style: share_bar_style(),
            span { style: share_label_style(), "{t(locale, TextKey::Share)}" }
            input {
                id: share_input_id,
                r#type: "text",
                readonly: true,
                value: "{share}",
                aria_label: "{t(locale, TextKey::CopyShareableLink)}",
                style: share_input_style(),
            }
            CopyButton {
                text: Arc::<str>::from(absolute_share_url(share)),
                title: t(locale, TextKey::CopyShareableLink),
                locale,
            }
        }
    }
}

/// Taxon-resolution warning notice.
#[component]
pub fn TaxonNotice() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let notice = use_result_selector(explore, |result| result.taxon_notice.clone());
    let notice = notice.read();
    let Some(warning) = notice.as_ref() else {
        return rsx! {};
    };
    let text = format_taxon_warning(locale, warning);
    rsx! {
        div { role: "status", style: notice_base_style(),
            span { style: notice_label_style(), "{t(locale, TextKey::Notice)}" }
            span { style: notice_value_style(), "{text}" }
        }
    }
}

/// Error notice with optional retry and dismiss buttons.
///
/// Retry visibility is delegated to `explore::recovery` so policy remains
/// consistent with orchestration-level error handling.
#[component]
pub fn ErrorNotice() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let interactions = use_explore_interactions();
    let retry_interactions = interactions.clone();
    let domain_error = use_lifecycle_selector(explore, |lifecycle| lifecycle.error.clone());
    let is_loading = use_lifecycle_selector(explore, |lifecycle| lifecycle.loading);
    let domain_error = domain_error.read();
    let Some(domain_err) = domain_error.as_ref() else {
        return rsx! {};
    };
    let kind: ErrorKind = domain_err.kind();
    let msg = format_domain_error(locale, domain_err);
    rsx! {
        div { role: "alert", style: notice_base_style(),
            span { style: notice_label_style(), "{t(locale, TextKey::Error)}" }
            span { style: notice_value_style(), "{msg}" }
            span { style: notice_value_style(), "{error_hint_text(locale, kind)}" }
            if recovery::should_show_retry_button(domain_err) && !*is_loading.read() {
                button {
                    r#type: "button",
                    style: button_base_style(),
                    onclick: move |_| retry_interactions.retry(),
                    "{t(locale, TextKey::Retry)}"
                }
            }
            button {
                r#type: "button",
                aria_label: "{t(locale, TextKey::DismissError)}",
                style: notice_dismiss_style(),
                onclick: move |_| interactions.dismiss_error(),
                "×"
            }
        }
    }
}

fn notice_base_style() -> String {
    StyleBuilder::new()
        .margin("10px 24px 0")
        .padding("9px 12px")
        .display("flex")
        .align_items("center")
        .gap("12px")
        .border_radius("var(--radius)")
        .font_size("var(--fs-0)")
        .border("1px solid var(--panel-border)")
        .background_color("var(--panel-bg-soft)")
        .box_shadow("var(--panel-shadow)")
        .property(
            "transition",
            "background .15s ease, border-color .15s ease, box-shadow .15s ease",
        )
        .build()
}

fn notice_label_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .property("text-transform", "uppercase")
        .property("letter-spacing", "1px")
        .font_size("var(--fs-label)")
        .font_weight("700")
        .property("line-height", "1.4")
        .padding("2px 6px")
        .border_radius("3px")
        .property("flex-shrink", "0")
        .build()
}

fn notice_value_style() -> String {
    StyleBuilder::new()
        .property("flex", "1")
        .color("var(--text)")
        .property("word-break", "break-word")
        .property("line-height", "1.4")
        .build()
}

fn notice_dismiss_style() -> String {
    StyleBuilder::new()
        .property("margin-left", "auto")
        .background_color("transparent")
        .border("0")
        .color("inherit")
        .cursor("pointer")
        .property("font-size", "18px")
        .property("line-height", "1")
        .padding("0 4px")
        .property("opacity", ".7")
        .build()
}

fn share_bar_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .property("flex-wrap", "wrap")
        .align_items("center")
        .gap("6px 10px")
        .margin("10px 24px 0")
        .padding("7px 12px")
        .border("1px solid var(--panel-border)")
        .border_radius("12px")
        .background_color("color-mix(in srgb, var(--panel-bg-soft) 92%, var(--surface))")
        .box_shadow("var(--panel-shadow)")
        .font_size("var(--fs-0)")
        .property(
            "transition",
            "background .15s ease, border-color .15s ease, box-shadow .15s ease",
        )
        .build()
}

fn share_label_style() -> String {
    StyleBuilder::new()
        .property("text-transform", "uppercase")
        .property("letter-spacing", "0.08em")
        .font_weight("700")
        .font_size("var(--fs-0)")
        .color("var(--text2)")
        .property("flex-shrink", "0")
        .property("white-space", "nowrap")
        .build()
}

fn share_input_style() -> String {
    StyleBuilder::new()
        .property("flex", "1")
        .property("min-width", "min(200px, 100%)")
        .background_color("var(--surface)")
        .border("1px solid var(--border)")
        .border_radius("var(--radius-sm)")
        .color("var(--text)")
        .padding("4px 8px")
        .font_size("var(--fs-0)")
        .build()
}

fn button_base_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .gap("6px")
        .border("1px solid var(--border)")
        .border_radius("4px")
        .property("min-height", "40px")
        .padding("8px 14px")
        .font_size("var(--fs-0)")
        .font_weight("600")
        .cursor("pointer")
        .background_color("var(--surface)")
        .color("var(--text)")
        .box_shadow("var(--shadow-xs)")
        .property(
            "transition",
            "background .15s, border-color .15s, box-shadow .15s, transform .12s ease",
        )
        .build()
}
