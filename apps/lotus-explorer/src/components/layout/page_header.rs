// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Page header: brand title, language switcher, view switcher, subtitle, archive note.
//!
//! Zero props -- all data comes from context (use_locale, AppStateContext).

use crate::components::layout::lang_switch::LangSwitch;
use crate::components::layout::view_switch::ViewSwitch;
use crate::hooks::use_locale;
use crate::i18n::{TextKey, t};
use dioxus::prelude::*;

/// Full page header section.
///
/// Composes `LangSwitch` and `ViewSwitch` as context-aware children.
/// Zero props -- only re-renders when locale or view changes.
#[component]
pub fn PageHeader() -> Element {
    let locale = use_locale();

    rsx! {
        div { class: "page-header",
            div { class: "page-brand",
                h1 { class: "page-title",
                    a {
                        class: "page-title-link",
                        href: "?",
                        title: "{t(locale, TextKey::PageTitle)}",
                        aria_label: "{t(locale, TextKey::PageTitle)}",
                        "{t(locale, TextKey::PageTitle)}"
                    }
                }
                LangSwitch {}
            }
            ViewSwitch {}
            p { class: "page-sub", "{t(locale, TextKey::PageSubtitle)}" }
            p { class: "page-archive-note",
                span { class: "page-archive-label", "{t(locale, TextKey::ArchiveNotice)}" }
                a {
                    class: "page-archive-link mono",
                    href: "https://doi.org/10.5281/zenodo.5794106",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "10.5281/zenodo.5794106"
                }
            }
        }
    }
}
