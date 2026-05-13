// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::components::copy_button::CopyButton;
use crate::features::explore::url_state::absolute_share_url;
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
