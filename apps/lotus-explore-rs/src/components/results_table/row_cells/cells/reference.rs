// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Reference identity cell for results-table rows.
//!
//! Renders the reference title (or QID fallback), Scholia link, DOI badge, and
//! Wikidata statement badge.

use crate::components::results_table::row_cells::prepared::PreparedRow;
use crate::components::results_table::row_cells::row_text::RowText;
use crate::i18n::{Locale, TextKey, aria_wikidata_statement, t};
use crate::models::CompoundEntry;
use crate::ui::classes;
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
        td { class: "{classes::TABLE_CELL_BASE} shadow-[inset_3px_0_0_var(--footer-wd-reference)]",
            div { class: "flex flex-col gap-1",
                if let Some(full_title) = entry.ref_title.as_deref() {
                    a {
                        href: "https://www.wikidata.org/entity/{reference_qid}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "block break-words line-clamp-2 font-semibold leading-snug hover:underline {classes::WD_REFERENCE}",
                        "{full_title}"
                    }
                } else {
                    a {
                        href: "https://www.wikidata.org/entity/{reference_qid}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "block break-words line-clamp-2 font-semibold leading-snug hover:underline {classes::WD_REFERENCE}",
                        "{reference_qid}"
                    }
                }
            }
            div { class: "mt-1 flex flex-wrap items-center gap-1",
                a {
                    href: "https://scholia.toolforge.org/work/{reference_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    aria_label: "{reference_qid} • {t(locale, TextKey::OpenInReferenceScholia)}",
                    class: "inline-block {classes::PILL} border-current {classes::WD_REFERENCE} {classes::WD_REFERENCE_BORDER}",
                    "{reference_qid} • Scholia"
                }
                if let Some(d) = doi {
                    a {
                        href: "https://doi.org/{d}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "inline-block {classes::PILL} border-current {classes::WD_REFERENCE} {classes::WD_REFERENCE_BORDER}",
                        aria_label: "{text.open_doi}",
                        "{d}"
                    }
                }
                if let Some(stmt) = statement_id {
                    a {
                        href: "https://www.wikidata.org/entity/statement/{stmt}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "inline-block {classes::PILL} border-current {classes::WD_REFERENCE} {classes::WD_REFERENCE_BORDER}",
                        aria_label: "{aria_wikidata_statement(locale, stmt)}",
                        "{stmt}"
                    }
                }
            }
        }
    }
}
