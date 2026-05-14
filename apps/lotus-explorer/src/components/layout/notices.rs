// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Status, warning, and error notice components.
//!
//! All notice components read locale via `use_locale()` and explore state via
//! `ResultsContext` — no `explore` or `locale` props are drilled from `App`.

use crate::components::copy_button::CopyButton;
use crate::features::explore::types::ErrorKind;
use crate::features::explore::url_state::absolute_share_url;
use crate::i18n::{TextKey, t};
use crate::services::error_presenter::{
    error_hint_text, format_domain_error, format_taxon_warning, is_retryable,
};
use crate::state::use_results_context;
use dioxus::prelude::*;
use std::sync::Arc;

/// Share URL notice — shows the current shareable URL with a copy button.
#[component]
pub fn ShareNotice(shareable_url: Memo<Option<Arc<str>>>) -> Element {
    let locale = crate::hooks::use_locale();
    let share = shareable_url.read();
    let Some(share) = share.as_deref() else {
        return rsx! {};
    };
    rsx! {
        div { class: "notice notice-info", role: "status",
            span { class: "notice-label", "{t(locale, TextKey::Share)}" }
            input {
                class: "notice-value notice-copy-field mono",
                r#type: "text",
                readonly: true,
                value: "{share}",
                aria_label: "{t(locale, TextKey::CopyShareableLink)}",
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
    let notice = explore.read().result.taxon_notice.clone();
    let Some(warning) = notice.as_ref() else {
        return rsx! {};
    };
    let text = format_taxon_warning(locale, warning);
    rsx! {
        div { class: "notice notice-warn", role: "status",
            span { class: "notice-label", "{t(locale, TextKey::Notice)}" }
            span { class: "notice-value", "{text}" }
        }
    }
}

/// Error notice with optional retry and dismiss buttons.
#[component]
pub fn ErrorNotice(on_dismiss: EventHandler<()>, on_retry: EventHandler<()>) -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let lifecycle = explore.read().lifecycle.clone();
    let Some(ref domain_err) = lifecycle.error else {
        return rsx! {};
    };
    let kind: ErrorKind = domain_err.kind();
    let is_loading = lifecycle.loading;
    let msg = format_domain_error(locale, domain_err);
    rsx! {
        div { class: "notice notice-error", role: "alert",
            span { class: "notice-label", "{t(locale, TextKey::Error)}" }
            span { class: "notice-value", "{msg}" }
            span { class: "notice-value", "{error_hint_text(locale, kind)}" }
            if is_retryable(kind) && !is_loading {
                button {
                    class: "btn btn-sm",
                    r#type: "button",
                    onclick: move |_| on_retry.call(()),
                    "{t(locale, TextKey::Retry)}"
                }
            }
            button {
                class: "notice-dismiss",
                r#type: "button",
                aria_label: "{t(locale, TextKey::DismissError)}",
                onclick: move |_| on_dismiss.call(()),
                "×"
            }
        }
    }
}

