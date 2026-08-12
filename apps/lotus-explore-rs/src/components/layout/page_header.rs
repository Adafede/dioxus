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
use crate::ui::style_constants::{spacing, text};
use dioxus::prelude::*;
use ui::prelude::*;

/// Full page header section.
///
/// Composes `LangSwitch` and `ViewSwitch` as context-aware children.
/// Zero props -- only re-renders when locale or view changes.
#[component]
pub fn PageHeader() -> Element {
    let locale = use_locale();

    rsx! {
        header { style: page_header_style(),
            div { style: page_brand_style(),
                h1 { id: PAGE_TITLE_ID, style: page_title_style(),
                    a {
                        href: "?",
                        aria_label: "{t(locale, TextKey::GoToHomepage)}",
                        style: page_title_link_style(),
                        span { style: page_title_text_style(), "{t(locale, TextKey::PageTitle)}" }
                    }
                }
                LangSwitch {}
            }
            ViewSwitch {}
            p { style: page_subtitle_style(),
                "{t(locale, TextKey::PageSubtitle)}"
                span { style: page_archive_note_style(),
                    " "
                    span { style: page_archive_label_style(), "{t(locale, TextKey::ArchiveNotice)}" }
                    a {
                        href: "https://doi.org/10.5281/zenodo.5794106",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        style: page_archive_link_style(),
                        "10.5281/zenodo.5794106"
                    }
                }
            }
        }
    }
}

fn page_header_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("8px")
        .property("padding-left", "max(18px, env(safe-area-inset-left))")
        .property("padding-right", "max(18px, env(safe-area-inset-right))")
        .build()
}

fn page_brand_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("row")
        .flex_wrap("wrap")
        .align_items("flex-start")
        .gap(spacing::PAGE_BRAND_GAP)
        .build()
}

fn page_title_style() -> String {
    StyleBuilder::new()
        .property("min-width", "0")
        .property("flex", "1 1 260px")
        .font_size("var(--fs-4)")
        .property("margin", "0")
        .build()
}

fn page_title_link_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .property("max-width", "100%")
        .gap("8px")
        .text_decoration("none")
        .color("inherit")
        .build()
}

fn page_title_text_style() -> String {
    StyleBuilder::new()
        .property("line-height", "1.1")
        .property("word-break", "break-word")
        .build()
}

fn page_subtitle_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-1)")
        .property("margin", "0")
        .color(text::SECONDARY)
        .build()
}

fn page_archive_note_style() -> String {
    StyleBuilder::new().display("inline").build()
}

fn page_archive_label_style() -> String {
    StyleBuilder::new()
        .font_weight("700")
        .property("font-variant", "small-caps")
        .build()
}

fn page_archive_link_style() -> String {
    StyleBuilder::new()
        .text_decoration("none")
        .color(text::ACCENT)
        .property("white-space", "nowrap")
        .build()
}
