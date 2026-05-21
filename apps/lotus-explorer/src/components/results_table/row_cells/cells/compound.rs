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

pub(in crate::components::results_table::row_cells) fn compound_cell(
    locale: Locale,
    text: RowText,
    entry: &CompoundEntry,
    prepared: &PreparedRow,
    name: &str,
    compound_qid: &str,
) -> Element {
    rsx! {
        td { class: "td-compound",
            div { class: "cell-primary",
                a {
                    href: "https://www.wikidata.org/entity/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "primary-link",
                    title: "{name}",
                    "{prepared.display_name}"
                }
            }
            div { class: "badge-row",
                a {
                    href: "https://www.wikidata.org/entity/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "id-badge wd",
                    title: "{text.open_in_wikidata}",
                    aria_label: "{aria_wikidata_entity(locale, compound_qid)}",
                    "{compound_qid}"
                }
                a {
                    href: "https://scholia.toolforge.org/chemical/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "id-badge sc",
                    title: "{text.open_in_scholia}",
                    aria_label: "{text.open_in_scholia}",
                    "Scholia"
                }
                if let Some(ik) = entry.inchikey.as_deref() {
                    a {
                        href: "https://www.wikidata.org/wiki/Special:Search?search={ik}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "id-badge mono inchikey",
                        title: "{ik}",
                        aria_label: "{aria_search_inchikey(locale, ik)}",
                        "{ik}"
                    }
                }
            }
        }
    }
}
