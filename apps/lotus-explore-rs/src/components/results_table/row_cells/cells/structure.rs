// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Structure (depiction image) cell for results-table rows.

use crate::components::results_table::row_cells::row_text::RowText;
use crate::i18n::{Locale, aria_chemical_structure};
use dioxus::prelude::*;
use ui::prelude::*;

pub(in crate::components::results_table::row_cells) fn structure_cell(
    locale: Locale,
    text: RowText,
    depict_url: Option<&str>,
    name: &str,
) -> Element {
    rsx! {
        td { style: structure_cell_style(),
            if let Some(url) = depict_url {
                a {
                    href: "{url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{text.open_full_size_depiction}",
                    img {
                        src: "{url}",
                        alt: "{aria_chemical_structure(locale, name)}",
                        loading: "lazy",
                        width: "120",
                        height: "72",
                        style: depict_img_style(),
                    }
                }
            } else {
                span { style: na_style(), "-" }
            }
        }
    }
}

fn na_style() -> String {
    StyleBuilder::new().color("var(--text3)").build()
}

fn structure_cell_style() -> String {
    StyleBuilder::new()
        .property("width", "auto")
        .property("min-width", "0")
        .padding("6px 10px")
        .build()
}

fn depict_img_style() -> String {
    StyleBuilder::new()
        .property("display", "block")
        .background_color("var(--bg2)")
        .border("1px solid var(--border)")
        .border_radius("6px")
        .property("width", "min(100%, 108px)")
        .property("max-width", "108px")
        .property("height", "auto")
        .property("object-fit", "contain")
        .box_shadow("var(--shadow-xs)")
        .build()
}
