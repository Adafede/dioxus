// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Taxon identity cell for results-table rows.

use crate::components::results_table::row_cells::row_text::RowText;
use crate::i18n::{Locale, aria_wikidata_entity};
use crate::models::CompoundEntry;
use crate::ui::style_constants::table_cells;
use dioxus::prelude::*;
use ui::prelude::*;

pub(in crate::components::results_table::row_cells) fn taxon_cell(
    locale: Locale,
    text: RowText,
    entry: &CompoundEntry,
    taxon_qid: &str,
) -> Element {
    rsx! {
        td { style: crate::ui::style_constants::table_cells::taxon_cell_style(),
            div { style: crate::ui::style_constants::table_cells::cell_primary_style(),
                a {
                    href: "https://www.wikidata.org/entity/{taxon_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    style: crate::ui::style_constants::table_cells::primary_link_style(),
                    "{entry.taxon_name}"
                }
            }
            div { style: crate::ui::style_constants::table_cells::badge_row_style(),
                a {
                    href: "https://www.wikidata.org/entity/{taxon_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{text.open_in_wikidata}",
                    aria_label: "{aria_wikidata_entity(locale, taxon_qid)}",
                    style: crate::ui::style_constants::table_cells::id_badge_style(),
                    "{taxon_qid}"
                }
            }
        }
    }
}
