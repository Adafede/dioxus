// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Taxon identity cell for results-table rows.
//!
//! Renders the taxon name link and Wikidata badge.

use crate::components::results_table::row_cells::row_text::RowText;
use crate::i18n::{Locale, aria_wikidata_entity};
use crate::models::CompoundEntry;
use dioxus::prelude::*;

pub(in crate::components::results_table::row_cells) fn taxon_cell(
    locale: Locale,
    text: RowText,
    entry: &CompoundEntry,
    taxon_qid: &str,
) -> Element {
    rsx! {
        td { class: "min-w-0 rounded-lg px-3 py-2.5 align-middle text-ui shadow-[inset_3px_0_0_var(--footer-wd-taxon)]",
            div { class: "flex flex-col gap-1",
                a {
                    href: "https://www.wikidata.org/entity/{taxon_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "block break-words line-clamp-2 font-semibold italic leading-snug hover:underline text-text",
                    "{entry.taxon_name}"
                }
            }
            div { class: "mt-1 flex flex-wrap items-center gap-1",
                a {
                    href: "https://www.wikidata.org/entity/{taxon_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{text.open_in_wikidata}",
                    aria_label: "{aria_wikidata_entity(locale, taxon_qid)}",
                    class: "inline-block rounded-xs border border-current px-2 py-0.5 font-mono text-micro font-semibold hover:underline text-text",
                    "{taxon_qid}"
                }
            }
        }
    }
}
