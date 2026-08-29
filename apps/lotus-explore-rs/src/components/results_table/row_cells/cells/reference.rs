// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Reference identity cell for results-table rows.
//!
//! Renders the reference title (or QID fallback), Wikidata badge, DOI badge, and
//! Wikidata statement badge.

use crate::components::results_table::row_cells::prepared::PreparedRow;
use crate::components::results_table::row_cells::row_text::RowText;
use crate::i18n::{Locale, aria_wikidata_entity, aria_wikidata_statement};
use crate::models::CompoundEntry;
use dioxus::prelude::*;

pub(in crate::components::results_table::row_cells) fn reference_cell(
    locale: Locale,
    text: RowText,
    entry: &CompoundEntry,
    prepared: &PreparedRow,
    reference_qid: &str,
) -> Element {
    let doi = prepared.doi.as_deref();
    let statement_id = prepared.statement_id.as_deref();
    rsx! {
        td { class: "min-w-0 rounded-lg px-3 py-2.5 align-middle text-ui shadow-[inset_3px_0_0_var(--footer-wd-reference)]",
            div { class: "flex flex-col gap-1",
                if let Some(full_title) = entry.ref_title.as_deref() {
                    a {
                        href: "https://www.wikidata.org/entity/{reference_qid}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "block break-words line-clamp-2 font-semibold leading-snug hover:underline text-wd-reference",
                        title: "{full_title}",
                        "{full_title}"
                    }
                } else {
                    a {
                        href: "https://www.wikidata.org/entity/{reference_qid}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "block break-words line-clamp-2 font-semibold leading-snug hover:underline text-wd-reference",
                        "{reference_qid}"
                    }
                }
            }
            div { class: "mt-1 flex flex-wrap items-center gap-1",
                a {
                    href: "https://www.wikidata.org/entity/{reference_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "inline-block rounded-xs border border-current px-2 py-0.5 font-mono text-micro font-semibold hover:underline text-wd-reference",
                    title: "{text.open_in_wikidata}",
                    aria_label: "{aria_wikidata_entity(locale, reference_qid)}",
                    "{reference_qid}"
                }
                if let Some(d) = doi {
                    a {
                        href: "https://doi.org/{d}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "inline-block rounded-xs border border-current px-2 py-0.5 font-mono text-micro font-semibold hover:underline text-wd-reference",
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
                        class: "inline-block rounded-xs border border-current px-2 py-0.5 font-mono text-micro font-semibold hover:underline text-wd-reference",
                        title: "{stmt}",
                        aria_label: "{aria_wikidata_statement(locale, stmt)}",
                        "{text.statement}"
                    }
                }
            }
        }
    }
}
