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
        td { class: "min-w-0 rounded-[10px] px-3 py-2 align-middle text-wd-compound shadow-[inset_3px_0_0_var(--footer-wd-compound)]",
            div { class: "flex flex-col gap-1",
                a {
                    href: "https://www.wikidata.org/entity/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{name}",
                    class: "block break-words font-semibold leading-snug hover:underline",
                    "{prepared.display_name}"
                }
            }
            div { class: "mt-1 flex flex-wrap items-center gap-1",
                a {
                    href: "https://www.wikidata.org/entity/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{text.open_in_wikidata}",
                    aria_label: "{aria_wikidata_entity(locale, compound_qid)}",
                    class: "inline-block rounded-[3px] border border-current px-1.5 py-0.5 font-mono text-micro font-semibold hover:underline",
                    "{compound_qid}"
                }
                a {
                    href: "https://scholia.toolforge.org/chemical/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{text.open_in_scholia}",
                    aria_label: "{text.open_in_scholia}",
                    class: "inline-block rounded-[3px] border border-current px-1.5 py-0.5 font-mono text-micro font-semibold hover:underline",
                    "Scholia"
                }
                if let Some(ik) = entry.inchikey.as_deref() {
                    a {
                        href: "https://www.wikidata.org/wiki/Special:Search?search={ik}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        title: "{ik}",
                        aria_label: "{aria_search_inchikey(locale, ik)}",
                        class: "inline-block rounded-[3px] border border-current px-1.5 py-0.5 font-mono text-micro font-semibold hover:underline",
                        "{ik}"
                    }
                }
            }
        }
    }
}
