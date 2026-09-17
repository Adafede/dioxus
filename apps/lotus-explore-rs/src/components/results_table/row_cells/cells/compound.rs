// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Compound identity cell for results-table rows.
//!
//! Renders the compound name link, Wikidata badge, Scholia link, and InChIKey badge.

use crate::components::results_table::row_cells::prepared::PreparedRow;
use crate::components::results_table::row_cells::row_text::RowText;
use crate::i18n::{Locale, TextKey, aria_search_inchikey, t};
use crate::models::CompoundEntry;
use dioxus::prelude::*;

pub(in crate::components::results_table::row_cells) fn compound_cell(
    locale: Locale,
    _text: RowText,
    entry: &CompoundEntry,
    prepared: &PreparedRow,
    _name: &str,
    compound_qid: &str,
) -> Element {
    rsx! {
        td { class: "min-w-0 rounded-xl px-3 py-2.5 align-middle text-ui shadow-[inset_2px_0_0_var(--border)]",
            div { class: "flex flex-col gap-1 max-w-[30ch]",
                a {
                    href: "https://www.wikidata.org/entity/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "block break-words hyphens-auto line-clamp-2 font-semibold leading-snug hover:underline text-wd-compound",
                    "{prepared.display_name}"
                }
            }
            div { class: "mt-1 flex flex-wrap items-center gap-1 max-w-[30ch]",
                a {
                    href: "https://scholia.toolforge.org/chemical/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    aria_label: "{compound_qid} • {t(locale, TextKey::OpenInCompoundScholia)}",
                    class: "inline-block inline-flex items-center rounded-full border border-panel-border bg-surface px-2 py-0.5 text-micro font-semibold uppercase tracking-wide border-current text-wd-compound border-wd-compound",
                    "{compound_qid} • Scholia"
                }
                if let Some(ik) = entry.inchikey.as_deref() {
                    a {
                        href: "https://www.wikidata.org/wiki/Special:Search?search={ik}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        aria_label: "{aria_search_inchikey(locale, ik)}",
                        class: "inline-block max-w-full break-all inline-flex items-center rounded-full border border-panel-border bg-surface px-2 py-0.5 text-micro font-semibold uppercase tracking-wide border-current text-wd-compound border-wd-compound",
                        "{ik}"
                    }
                }
            }
        }
    }
}
