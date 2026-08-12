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
        NoticeBar {
            label: t(locale, TextKey::Share).to_string(),
            tone: NoticeTone::Info,
            role: "status",
            aria_live: "polite",
            dark: true,
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
        NoticeBar {
            label: t(locale, TextKey::Notice).to_string(),
            tone: NoticeTone::Warning,
            role: "status",
            aria_live: "polite",
            dark: true,
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
        NoticeBar {
            label: t(locale, TextKey::Error).to_string(),
            tone: NoticeTone::Danger,
            role: "alert",
            aria_live: "assertive",
            dark: true,
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

fn notice_value_style() -> String {
    crate::ui::style_constants::shared::notice_value_style()
}

fn button_base_style() -> String {
    crate::ui::style_constants::buttons::button_base_style()
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

fn share_input_style() -> String {
    StyleBuilder::new()
        .property("flex", "1 1 200px")
        .property("min-width", "min(200px, 100%)")
        .background_color("var(--surface)")
        .border("1px solid var(--border)")
        .border_radius("var(--radius-sm)")
        .color("var(--text)")
        .padding("4px 8px")
        .font_size("var(--fs-0)")
        .build()
}
