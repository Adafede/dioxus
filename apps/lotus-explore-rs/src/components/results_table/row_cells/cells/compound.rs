// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Compound identity cell for results-table rows.
//!
//! Renders the compound name link, Wikidata badge, Scholia badge, and InChIKey badge.

use crate::components::results_table::row_cells::prepared::PreparedRow;
use crate::components::results_table::row_cells::row_text::RowText;
use crate::i18n::{Locale, aria_search_inchikey, aria_wikidata_entity};
use crate::models::CompoundEntry;
use dioxus::prelude::*;
use ui::prelude::*;
use ui::styles::lotus::tokens::FOOTER_WD_COMPOUND;

pub(in crate::components::results_table::row_cells) fn compound_cell(
    locale: Locale,
    text: RowText,
    entry: &CompoundEntry,
    prepared: &PreparedRow,
    name: &str,
    compound_qid: &str,
) -> Element {
    rsx! {
        td { style: compound_cell_style(),
            div { style: cell_primary_style(),
                a {
                    href: "https://www.wikidata.org/entity/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{name}",
                    style: primary_link_style(),
                    "{prepared.display_name}"
                }
            }
            div { style: badge_row_style(),
                a {
                    href: "https://www.wikidata.org/entity/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{text.open_in_wikidata}",
                    aria_label: "{aria_wikidata_entity(locale, compound_qid)}",
                    style: id_badge_style("transparent", FOOTER_WD_COMPOUND, FOOTER_WD_COMPOUND),
                    "{compound_qid}"
                }
                a {
                    href: "https://scholia.toolforge.org/chemical/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{text.open_in_scholia}",
                    aria_label: "{text.open_in_scholia}",
                    style: id_badge_style("transparent", FOOTER_WD_COMPOUND, FOOTER_WD_COMPOUND),
                    "Scholia"
                }
                if let Some(ik) = entry.inchikey.as_deref() {
                    a {
                        href: "https://www.wikidata.org/wiki/Special:Search?search={ik}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        title: "{ik}",
                        aria_label: "{aria_search_inchikey(locale, ik)}",
                        style: id_badge_style("transparent", FOOTER_WD_COMPOUND, FOOTER_WD_COMPOUND),
                        "{ik}"
                    }
                }
            }
        }
    }
}

fn compound_cell_style() -> String {
    StyleBuilder::new()
        .padding("8px 12px")
        .border_radius("10px")
        .background_color("transparent")
        .color(FOOTER_WD_COMPOUND)
        .property(
            "box-shadow",
            &format!("inset 3px 0 0 {}", FOOTER_WD_COMPOUND),
        )
        .property("min-width", "0")
        .build()
}

fn cell_primary_style() -> String {
    StyleBuilder::new().font_weight("500").build()
}

fn id_badge_style(_bg: &str, fg: &str, border: &str) -> String {
    StyleBuilder::new()
        .display("inline-block")
        .font_size(FS_MICRO)
        .padding("1px 5px")
        .border_radius("3px")
        .font_weight("600")
        .text_decoration("none")
        .property("line-height", "1.5")
        .border("1px solid transparent")
        .font_family(FONT_MONO)
        .property("max-width", "100%")
        .property("white-space", "normal")
        .property("overflow-wrap", "anywhere")
        .property(
            "transition",
            "transform .12s ease, box-shadow .12s ease, filter .12s ease",
        )
        .background_color("transparent")
        .color(fg)
        .border("1px solid")
        .property("border-color", border)
        .build()
}

fn primary_link_style() -> String {
    StyleBuilder::new()
        .color("inherit")
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
