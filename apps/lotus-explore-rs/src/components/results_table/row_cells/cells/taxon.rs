// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Taxon identity cell for results-table rows.
//!
//! Renders the taxon name link and Scholia/Wikidata badges.

use crate::components::results_table::row_cells::row_text::RowText;
use crate::i18n::{Locale, TextKey, t};
use crate::models::CompoundEntry;
use crate::ui::classes;
use dioxus::prelude::*;

pub(in crate::components::results_table::row_cells) fn taxon_cell(
    _locale: Locale,
    _text: RowText,
    entry: &CompoundEntry,
    taxon_qid: &str,
) -> Element {
    rsx! {
        td { class: "{classes::TABLE_CELL_BASE} shadow-[inset_3px_0_0_var(--footer-wd-taxon)]",
            div { class: "flex flex-col gap-1 max-w-[24ch]",
                a {
                    href: "https://www.wikidata.org/entity/{taxon_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "block break-words line-clamp-2 font-semibold italic leading-snug hover:underline {classes::WD_TAXON}",
                    "{entry.taxon_name}"
                }
            }
            div { class: "mt-1 flex flex-wrap items-center gap-1 max-w-[24ch]",
                a {
                    href: "https://scholia.toolforge.org/taxon/{taxon_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    aria_label: "{taxon_qid} • {t(_locale, TextKey::OpenInTaxonScholia)}",
                    class: "inline-block {classes::PILL} border-current {classes::WD_TAXON} border-wd-taxon",
                    "{taxon_qid} • Scholia"
                }
            }
        }
    }
}
