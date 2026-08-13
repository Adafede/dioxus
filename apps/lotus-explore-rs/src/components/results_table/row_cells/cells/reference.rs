// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Reference (publication) cell for results-table rows.
//!
//! Renders the reference title link, Wikidata badge, DOI badge, and statement badge.

use crate::components::results_table::row_cells::prepared::PreparedRow;
use crate::components::results_table::row_cells::row_text::RowText;
use crate::i18n::{Locale, aria_wikidata_entity, aria_wikidata_statement};
use crate::models::CompoundEntry;
use dioxus::prelude::*;
use ui::prelude::*;

pub(in crate::components::results_table::row_cells) fn reference_cell(
    locale: Locale,
    text: RowText,
    entry: &CompoundEntry,
    _prepared: &PreparedRow,
    reference_qid: &str,
    doi: Option<&str>,
    statement_id: Option<&str>,
) -> Element {
    rsx! {
    td { style: crate::ui::style_constants::table_cells::reference_cell_style(),
        div { style: cell_primary_style(),
                if let Some(full_title) = entry.ref_title.as_deref()
                {
                    a {
                        href: "https://www.wikidata.org/entity/{reference_qid}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                    style: crate::ui::style_constants::table_cells::primary_link_style(),
                        title: "{full_title}",
                        "{full_title}"
                    }
                } else {
                    a {
                        href: "https://www.wikidata.org/entity/{reference_qid}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                    style: crate::ui::style_constants::table_cells::primary_link_style(),
                        "{reference_qid}"
                    }
                }
            }
        div { style: crate::ui::style_constants::table_cells::badge_row_style(),
                a {
                    href: "https://www.wikidata.org/entity/{reference_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                style: id_badge_style("var(--wd-reference-soft-bg)", "var(--wd-reference)", "var(--wd-reference-soft-border)"),
                    title: "{text.open_in_wikidata}",
                    aria_label: "{aria_wikidata_entity(locale, reference_qid)}",
                    "{reference_qid}"
                }
                if let Some(d) = doi {
                    a {
                        href: "https://doi.org/{d}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                    style: id_badge_style("var(--wd-reference-soft-bg)", "var(--wd-reference)", "var(--wd-reference-soft-border-weak)"),
                        title: "{text.open_doi}",
                        aria_label: "{text.open_doi}",
                        "DOI"
                    }
                }
                if let Some(stmt) = statement_id {
                    a {
                        href: "https://www.wikidata.org/entity/statement/{stmt}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        style: id_badge_style("var(--wd-reference-soft-bg)", "var(--wd-reference)", "var(--wd-reference-soft-border-weak)"),
                        title: "{stmt}",
                        aria_label: "{aria_wikidata_statement(locale, stmt)}",
                        "{text.statement}"
                    }
                }
            }
        }
    }
}
fn cell_primary_style() -> String {
    StyleBuilder::new().font_weight("500").build()
}
fn id_badge_style(bg: &str, fg: &str, border: &str) -> String {
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
        .background_color(bg)
        .color(fg)
        .border("1px solid")
        .property("border-color", border)
        .build()
}
