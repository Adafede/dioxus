// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Page header: brand title, language switcher, view switcher, subtitle, archive note.
//!
//! Zero props -- all data comes from context (use_locale, AppStateContext).

use crate::components::layout::lang_switch::LangSwitch;
use crate::components::layout::view_switch::ViewSwitch;
use crate::hooks::use_locale;
use crate::i18n::{TextKey, t};
use crate::ui::a11y_contract::PAGE_TITLE_ID;
use crate::ui::style_constants::header;
use dioxus::prelude::*;

/// Full page header section.
///
/// Composes `LangSwitch` and `ViewSwitch` as context-aware children.
/// Zero props -- only re-renders when locale or view changes.
#[component]
pub fn PageHeader() -> Element {
    let locale = use_locale();

    rsx! {
        header { style: header::page_header_style(),
            div { style: header::page_brand_style(),
                h1 { id: PAGE_TITLE_ID, style: header::page_title_style(),
                    a {
                        href: "?",
                        class: "page-title-link page-home-link",
                        aria_label: "{t(locale, TextKey::GoToHomepage)}",
                        style: header::page_title_link_style(),
                        span { style: header::page_title_text_style(), "{t(locale, TextKey::PageTitle)}" }
                    }
                }
                LangSwitch {}
            }
            ViewSwitch {}
            p { style: header::page_subtitle_style(),
                "{t(locale, TextKey::PageSubtitle)}"
                span { style: header::page_archive_note_style(),
                    span { style: header::page_archive_label_style(), "{t(locale, TextKey::ArchiveNotice)}" }
                    a {
                        href: "https://doi.org/10.5281/zenodo.5794106",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        style: header::page_archive_link_style(),
                        "10.5281/zenodo.5794106"
                    }
                }
            }
        }
    }
}
