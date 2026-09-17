// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Structure depiction cell for results-table rows.
//!
//! Renders a depiction image (lazy-loaded) when available, otherwise a dash.

use crate::components::results_table::row_cells::row_text::RowText;
use crate::i18n::{Locale, aria_chemical_structure};
use crate::ui::classes;
use dioxus::prelude::*;

/// Truncate alt text to avoid overly long alternative text (>100 chars)
fn truncate_alt(text: &str, max: usize) -> String {
    if text.len() <= max {
        text.to_string()
    } else {
        format!("{}...", &text[..max.saturating_sub(3)])
    }
}

pub(in crate::components::results_table::row_cells) fn structure_cell(
    locale: Locale,
    _text: RowText,
    depict_url: Option<std::sync::Arc<str>>,
    name: &str,
) -> Element {
    let alt_text = truncate_alt(&aria_chemical_structure(locale, name), 100);
    rsx! {
        td { class: "{classes::TABLE_CELL_BASE} {classes::WD_STRUCTURE} border-l-2 border-l-wd-structure",
            if let Some(url) = depict_url {
                a {
                    href: "{url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                        img {
                        class: "block h-auto w-full min-w-[120px] bg-transparent object-contain",
                            src: "{url}",
                            alt: "{alt_text}",
                            loading: "lazy",
                            decoding: "async",
                    }
                }
            } else {
                span { class: "text-subtle", "-" }
            }
        }
    }
}
