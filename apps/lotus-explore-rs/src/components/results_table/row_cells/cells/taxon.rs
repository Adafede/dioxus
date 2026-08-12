// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Taxon identity cell for results-table rows.

use crate::components::results_table::row_cells::row_text::RowText;
use crate::i18n::{Locale, aria_wikidata_entity};
use crate::models::CompoundEntry;
use dioxus::prelude::*;
use ui::prelude::*;

pub(in crate::components::results_table::row_cells) fn taxon_cell(
    locale: Locale,
    text: RowText,
    entry: &CompoundEntry,
    taxon_qid: &str,
) -> Element {
    rsx! {
        td { style: taxon_cell_style(),
            div { style: cell_primary_style(),
                a {
                    href: "https://www.wikidata.org/entity/{taxon_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    style: primary_link_style(),
                    "{entry.taxon_name}"
                }
            }
            div { style: badge_row_style(),
                a {
                    href: "https://www.wikidata.org/entity/{taxon_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{text.open_in_wikidata}",
                    aria_label: "{aria_wikidata_entity(locale, taxon_qid)}",
                    style: id_badge_style(),
                    "{taxon_qid}"
                }
            }
        }
    }
}

fn taxon_cell_style() -> String {
    StyleBuilder::new()
        .padding("8px 12px")
        .border_radius("10px")
        .background_color("color-mix(in srgb, var(--surface) 90%, transparent)")
        .property(
            "box-shadow",
            "inset 3px 0 0 rgb(51 153 102 / 42%), inset 0 0 0 1px var(--results-border)",
        )
        .property("min-width", "0")
        .build()
}

fn cell_primary_style() -> String {
    StyleBuilder::new()
        .font_weight("500")
        .property("font-style", "italic")
        .build()
}

fn id_badge_style() -> String {
    StyleBuilder::new()
        .display("inline-block")
        .font_size("var(--fs-micro)")
        .padding("1px 5px")
        .border_radius("3px")
        .font_weight("600")
        .text_decoration("none")
        .property("line-height", "1.5")
        .border("1px solid transparent")
        .font_family("var(--mono)")
        .property("max-width", "100%")
        .property("white-space", "normal")
        .property("overflow-wrap", "anywhere")
        .property(
            "transition",
            "transform .12s ease, box-shadow .12s ease, filter .12s ease",
        )
        .background_color("var(--wd-taxon-soft-bg)")
        .color("var(--wd-taxon)")
        .property("border-color", "var(--wd-taxon-soft-border)")
        .build()
}

fn primary_link_style() -> String {
    StyleBuilder::new()
        .color("var(--text)")
        .property("display", "block")
        .property("line-height", "1.4")
        .property("overflow-wrap", "break-word")
        .property("word-break", "break-word")
        .property("white-space", "normal")
        .build()
}

fn badge_row_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .property("flex-wrap", "wrap")
        .gap("4px")
        .property("margin-top", "4px")
        .property("overflow", "visible")
        .property("min-width", "0")
        .build()
}
