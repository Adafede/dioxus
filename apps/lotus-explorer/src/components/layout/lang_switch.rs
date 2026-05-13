// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Language-switcher button group.
//!
//! Reads and writes the `Signal<Locale>` from `LocaleProvider` context via
//! [`use_locale_signal`] — zero props required.

use crate::hooks::{use_locale, use_locale_signal};
use crate::i18n::{Locale, TextKey, t};
use dioxus::prelude::*;

/// Four-button language switcher (EN / FR / DE / IT).
///
/// Zero props — reads and writes the locale signal from `LocaleProvider`
/// context.  Only re-renders when the locale changes.
#[component]
pub fn LangSwitch() -> Element {
    let mut locale_sig = use_locale_signal();
    let locale = use_locale();

    rsx! {
        div {
            class: "lang-switch",
            role: "group",
            aria_label: "{t(locale, TextKey::Language)}",
            LangBtn {
                code: "EN",
                target: Locale::En,
                current: locale,
                on_select: move |l| {
                    if *locale_sig.peek() != l {
                        *locale_sig.write() = l;
                    }
                },
            }
            LangBtn {
                code: "FR",
                target: Locale::Fr,
                current: locale,
                on_select: move |l| {
                    if *locale_sig.peek() != l {
                        *locale_sig.write() = l;
                    }
                },
            }
            LangBtn {
                code: "DE",
                target: Locale::De,
                current: locale,
                on_select: move |l| {
                    if *locale_sig.peek() != l {
                        *locale_sig.write() = l;
                    }
                },
            }
            LangBtn {
                code: "IT",
                target: Locale::It,
                current: locale,
                on_select: move |l| {
                    if *locale_sig.peek() != l {
                        *locale_sig.write() = l;
                    }
                },
            }
        }
    }
}

/// Single language button.
#[component]
fn LangBtn(
    code: &'static str,
    target: Locale,
    current: Locale,
    on_select: EventHandler<Locale>,
) -> Element {
    let active = current == target;
    rsx! {
        button {
            class: if active { "btn btn-xs lang-btn active" } else { "btn btn-xs lang-btn" },
            r#type: "button",
            aria_pressed: if active { "true" } else { "false" },
            onclick: move |_| on_select.call(target),
            "{code}"
        }
    }
}
