// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::components::copy_button::CopyButton;
use crate::features::explore::types::ErrorKind;
use crate::features::explore::url_state::absolute_share_url;
#[cfg(target_arch = "wasm32")]
use crate::i18n::error_hint_memory;
use crate::i18n::{Locale, TextKey, t};
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn ShareNotice(shareable_url: Memo<Option<Arc<str>>>, locale: Signal<Locale>) -> Element {
    let locale = *locale.read();
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

#[component]
pub fn TaxonNotice(taxon_notice: Signal<Option<String>>, locale: Signal<Locale>) -> Element {
    let locale = *locale.read();
    let notice = taxon_notice.read();
    let Some(warning) = notice.as_deref() else {
        return rsx! {};
    };
    rsx! {
        div { class: "notice notice-warn", role: "status",
            span { class: "notice-label", "{t(locale, TextKey::Notice)}" }
            span { class: "notice-value", "{warning}" }
        }
    }
}

// ── Error notice ──────────────────────────────────────────────────────────────

#[component]
pub fn ErrorNotice(
    error: Signal<Option<String>>,
    error_kind: Signal<ErrorKind>,
    locale: Signal<Locale>,
    loading: Signal<bool>,
    on_dismiss: EventHandler<()>,
    on_retry: EventHandler<()>,
) -> Element {
    let locale = *locale.read();
    let kind = *error_kind.read();
    let is_loading = *loading.read();
    let err_ref = error.read();
    let Some(msg) = err_ref.as_deref() else {
        return rsx! {};
    };
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

// ── Pure helpers ──────────────────────────────────────────────────────────────

pub fn is_retryable(kind: ErrorKind) -> bool {
    matches!(
        kind,
        ErrorKind::Network | ErrorKind::Parse | ErrorKind::Unknown
    )
}

pub fn error_hint_text(locale: Locale, kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::Validation => t(locale, TextKey::ErrorHintValidation),
        ErrorKind::Network => t(locale, TextKey::ErrorHintNetwork),
        ErrorKind::Parse => t(locale, TextKey::ErrorHintParse),
        #[cfg(target_arch = "wasm32")]
        ErrorKind::Memory => error_hint_memory(locale),
        ErrorKind::Unknown => t(locale, TextKey::ErrorHintUnknown),
    }
}
