// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::curation::{CurationResultRow, CurationStatus};
use crate::i18n::{
    Locale, TextKey, col_canonical_smiles, col_exact_mass, col_name, col_original_smiles,
    col_status, curation_badge_mass_missing, curation_badge_prerequisite_pending,
    curation_badge_second_pass_required, curation_mass_warning_title, curation_status_label,
    hint_scroll_curation_results, label_new_item, t,
};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

const NA_TEXT: &str = "n/a";

const PILL: &str = "inline-flex items-center rounded-md border border-panel-border bg-surface px-2 py-0.5 text-xs font-semibold uppercase tracking-wide";
const TH: &str = "border-b border-panel-border bg-panel-soft px-3 py-2 text-left text-xs font-semibold uppercase tracking-wide text-muted";
const TD: &str = "border-b border-panel-border px-3 py-2 align-top text-ui";
const MONO: &str = "font-mono text-xs break-all";
const MONO_NB: &str = "font-mono text-xs";

#[component]
fn StatusSummaryBadges(locale: Locale, rows: Arc<[CurationResultRow]>) -> Element {
    rsx! {
        div { class: "flex flex-wrap gap-1.5",
            for (status, count) in status_counts(rows.as_ref()) {
                span { class: "{PILL} {status_text_class(&status)}",
                    "{status_label(locale, &status)} ({count})"
                }
            }
        }
    }
}

fn render_curation_result_cells(locale: Locale, row: &CurationResultRow) -> Element {
    rsx! {
        td { class: TD,
            span { class: "{PILL} {status_text_class(&row.status)}",
                "{status_label(locale, &row.status)}"
            }
            div { class: "mt-1 flex flex-wrap gap-1",
                if !row.dependency_blocks.is_empty() {
                    span { class: "{PILL} {status_text_class(&CurationStatus::PendingDependencies)}",
                        "{curation_badge_prerequisite_pending(locale)}"
                    }
                }
                if matches!(row.status, CurationStatus::PendingDependencies) {
                    span { class: "{PILL} {status_text_class(&CurationStatus::PendingDependencies)}",
                        "{curation_badge_second_pass_required(locale)}"
                    }
                }
                if row.exact_mass.is_none() {
                    span {
                        class: "{PILL} text-wd-entries",
                        title: "{row.mass_warning.as_deref().unwrap_or(curation_mass_warning_title(locale))}",
                        "{curation_badge_mass_missing(locale)}"
                    }
                }
            }
            if !row.note.is_empty() {
                div { class: "mt-1 whitespace-pre-line text-xs text-muted", "{row.note}" }
            }
        }
        td { class: TD,
            if let Some(qid) = row.wikidata_qid.as_deref() {
                a {
                    class: "font-mono text-xs text-wd-compound underline",
                    href: "https://www.wikidata.org/wiki/{qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "{qid}"
                }
            } else {
                span { class: "text-muted", "{label_new_item(locale)}" }
            }
        }
        td { class: TD, "{row.input.name}" }
        td { class: "{TD} {MONO}", "{row.input.smiles}" }
        td { class: "{TD} {MONO}", "{row.canonical_smiles.as_deref().unwrap_or(NA_TEXT)}" }
        td { class: "{TD} {MONO_NB}", "{row.inchikey.as_deref().unwrap_or(NA_TEXT)}" }
        td { class: "{TD} {MONO}", "{row.inchi.as_deref().unwrap_or(NA_TEXT)}" }
        td { class: "{TD} {MONO_NB}", "{row.formula.as_deref().unwrap_or(NA_TEXT)}" }
        td { class: "{TD} {MONO_NB}", "{format_mass(row.exact_mass)}" }
    }
}

#[component]
pub fn CurationResultsTable(locale: Locale, rows: Arc<[CurationResultRow]>) -> Element {
    let scroll_hint_id = "curation-results-scroll-hint";

    rsx! {
        div { class: "flex flex-col gap-3 rounded-xl border border-panel-border bg-panel-soft p-4 shadow-xs",
            h3 { class: "text-base font-semibold", "{crate::i18n::heading_results(locale)}" }
            StatusSummaryBadges { locale, rows: rows.clone() }
            p {
                id: scroll_hint_id,
                class: "inline-flex items-center gap-2 text-xs text-muted",
                span { class: "text-sm font-bold text-accent", "↔" }
                "{hint_scroll_curation_results(locale)}"
            }
            div {
                class: "w-full overflow-x-auto rounded-lg border border-panel-border focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40",
                role: "region",
                tabindex: "0",
                aria_label: "{crate::i18n::heading_results(locale)}",
                aria_describedby: scroll_hint_id,
                table { class: "w-full table-fixed border-collapse text-ui",
                    thead {
                        tr { class: "text-left",
                            th { scope: "col", class: "{TH} min-w-[170px]", "{col_status(locale)}" }
                            th { scope: "col", class: "{TH} min-w-[8ch]", "Wikidata" }
                            th { scope: "col", class: "{TH} min-w-[14ch]", "{col_name(locale)}" }
                            th { scope: "col", class: "{TH} min-w-[160px]", "{col_original_smiles(locale)}" }
                            th { scope: "col", class: "{TH} min-w-[160px]", "{col_canonical_smiles(locale)}" }
                            th { scope: "col", class: "{TH} min-w-[180px]", "InChIKey" }
                            th { scope: "col", class: "{TH} min-w-[160px]", "InChI" }
                            th { scope: "col", class: "{TH} min-w-[8ch]", "{t(locale, TextKey::Formula)}" }
                            th { scope: "col", class: "{TH} min-w-[8ch]", "{col_exact_mass(locale)}" }
                        }
                    }
                    tbody {
                        for (idx, row) in rows.iter().enumerate() {
                            tr { key: "{row.inchikey.as_deref().unwrap_or(&idx.to_string())}",
                                class: "odd:bg-surface/30 hover:bg-surface/60",
                                {render_curation_result_cells(locale, row)}
                            }
                        }
                    }
                }
            }
        }
    }
}

fn status_label(locale: Locale, status: &CurationStatus) -> &'static str {
    let key = match status {
        CurationStatus::ExistingComplete => "existing_complete",
        CurationStatus::ExistingNeedsUpdates => "existing_updates",
        CurationStatus::NewCompound => "new_compound",
        CurationStatus::PendingDependencies => "pending_dependencies",
        CurationStatus::Error => "error",
    };
    curation_status_label(locale, key)
}

fn status_text_class(status: &CurationStatus) -> &'static str {
    match status {
        CurationStatus::ExistingComplete => "text-wd-taxon",
        CurationStatus::ExistingNeedsUpdates => "text-wd-entries",
        CurationStatus::NewCompound => "text-wd-reference",
        CurationStatus::PendingDependencies => "text-wd-reference",
        CurationStatus::Error => "text-wd-compound",
    }
}

fn status_counts(rows: &[CurationResultRow]) -> Vec<(CurationStatus, usize)> {
    let mut counts = HashMap::<CurationStatus, usize>::new();
    for row in rows {
        *counts.entry(row.status.clone()).or_insert(0) += 1;
    }

    let ordered = [
        CurationStatus::ExistingComplete,
        CurationStatus::ExistingNeedsUpdates,
        CurationStatus::NewCompound,
        CurationStatus::PendingDependencies,
        CurationStatus::Error,
    ];

    ordered
        .into_iter()
        .filter_map(|status| counts.get(&status).copied().map(|count| (status, count)))
        .collect::<Vec<_>>()
}

fn format_mass(value: Option<f64>) -> String {
    value.map_or_else(|| "n/a".to_string(), |m| format!("{m:.5}"))
}
