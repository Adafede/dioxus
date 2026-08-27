// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Page header: brand title, language switcher, view switcher, subtitle, archive note.
//!
//! Zero props -- all data comes from context (use_locale, AppStateContext).

use crate::components::layout::dark_mode_toggle::DarkModeToggle;
use crate::components::layout::lang_switch::LangSwitch;
use crate::components::layout::view_switch::ViewSwitch;
use crate::hooks::use_locale;
use crate::i18n::{TextKey, t};
use crate::ui::a11y_contract::PAGE_TITLE_ID;
use dioxus::prelude::*;

/// Full page header section.
///
/// Composes `LangSwitch` (EN/FR/DE/IT), `DarkModeToggle` (light/dark), and
/// `ViewSwitch` (Search / Curation / Structure editor) as context-aware
/// children. Zero props -- only re-renders when locale or view changes.
#[component]
pub fn PageHeader() -> Element {
    let locale = use_locale();

    rsx! {
        header {
            class: "page-header",
            div {
                class: "page-brand",
                h1 { id: PAGE_TITLE_ID,
                    class: "page-title-text",
                    a {
                        href: "/dioxus/lotus-explore-rs/",
                        class: "page-title-link page-home-link",
                        aria_label: "{t(locale, TextKey::GoToHomepage)}",
                        span {
                            class: "page-title",
                            "{t(locale, TextKey::PageTitle)}"
                        }
                    }
                }
                div {
                    class: "header-controls",
                    ViewSwitch {}
                    LangSwitch {}
                    DarkModeToggle {}
                }
            }
            p {
                class: "page-subtitle",
                "{t(locale, TextKey::PageSubtitle)}"
                span {
                    class: "page-archive-note",
                    span {
                        class: "page-archive-label",
                        "{t(locale, TextKey::ArchiveNotice)}"
                    }
                    a {
                        href: "https://doi.org/10.5281/zenodo.5794106",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "page-archive-link border-b-2 border-current underline-offset-2 hover:no-underline",
                        "10.5281/zenodo.5794106"
                    }
                }
            }
        }
    }
}
