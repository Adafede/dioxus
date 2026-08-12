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
        .padding("14px 24px 10px")
        .border_bottom("1px solid var(--panel-border)")
        .background_color("color-mix(in srgb, var(--panel-bg-soft) 92%, var(--surface))")
        .box_shadow("var(--shadow-xs)")
        .property("position", "sticky")
        .property("top", "0")
        .property("z-index", "3")
        .property("overflow", "clip")
        .build()
}

fn page_brand_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .align_items("center")
        .gap("12px")
        .build()
}

fn page_title_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-4)")
        .font_weight("800")
        .property("letter-spacing", "-.028em")
        .property("line-height", "1.06")
        .color("var(--text)")
        .build()
}

fn page_title_link_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .text_decoration("none")
        .color("inherit")
        .build()
}

fn page_title_text_style() -> String {
    StyleBuilder::new()
        .property("min-width", "0")
        .property("overflow-wrap", "anywhere")
        .build()
}

fn page_subtitle_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-1)")
        .color("var(--critical-muted)")
        .property("margin-top", "4px")
        .build()
}

fn page_archive_note_style() -> String {
    StyleBuilder::new()
        .display("inline")
        .property("margin-left", "0.4em")
        .build()
}

fn page_archive_label_style() -> String {
    StyleBuilder::new()
        .font_size("inherit")
        .property("text-transform", "none")
        .property("letter-spacing", "normal")
        .color("var(--text2)")
        .font_weight("500")
        .property("margin-right", "0.25em")
        .build()
}

fn page_archive_link_style() -> String {
    StyleBuilder::new()
        .color("var(--accent)")
        .font_weight("500")
        .build()
}
