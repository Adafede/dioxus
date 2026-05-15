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

#[component]
pub fn CurationResultsTable(locale: Locale, rows: Arc<[CurationResultRow]>) -> Element {
    rsx! {
        div { class: "curation-card",
            h3 { "{crate::i18n::heading_results(locale)}" }
            div { class: "curation-badges",
                for (status, count) in status_counts(rows.as_ref()) {
                    span { class: "curation-status curation-status-badge {status_class(&status)}",
                        "{status_label(locale, &status)} ({count})"
                    }
                }
            }
            p { class: "curation-scroll-hint", "{hint_scroll_curation_results(locale)}" }
            div { class: "curation-table-scroll",
                table { class: "curation-table curation-results-table",
                    thead {
                        tr {
                            th { "{col_name(locale)}" }
                            th { "{col_original_smiles(locale)}" }
                            th { "{col_canonical_smiles(locale)}" }
                            th { "InChIKey" }
                            th { "InChI" }
                            th { "{t(locale, TextKey::Formula)}" }
                            th { "{col_exact_mass(locale)}" }
                            th { "Wikidata" }
                            th { "{col_status(locale)}" }
                        }
                    }
                    tbody {
                        for row in rows.iter() {
                            tr {
                                td { "{row.input.name}" }
                                td { class: "mono curation-cell-wrap", "{row.input.smiles}" }
                                td { class: "mono curation-cell-wrap",
                                    "{row.canonical_smiles.as_deref().unwrap_or(NA_TEXT)}"
                                }
                                td { class: "mono", "{row.inchikey.as_deref().unwrap_or(NA_TEXT)}" }
                                td { class: "mono curation-cell-wrap",
                                    "{row.inchi.as_deref().unwrap_or(NA_TEXT)}"
                                }
                                td { class: "mono", "{row.formula.as_deref().unwrap_or(NA_TEXT)}" }
                                td { class: "mono", "{format_mass(row.exact_mass)}" }
                                td {
                                    if let Some(qid) = row.wikidata_qid.as_deref() {
                                        a {
                                            href: "https://www.wikidata.org/wiki/{qid}",
                                            target: "_blank",
                                            rel: "noopener noreferrer",
                                            "{qid}"
                                        }
                                    } else {
                                        "{label_new_item(locale)}"
                                    }
                                }
                                td {
                                    span { class: "curation-status {status_class(&row.status)}",
                                        "{status_label(locale, &row.status)}"
                                    }
                                    div { class: "curation-row-badges",
                                        if !row.dependency_blocks.is_empty() {
                                            span { class: "curation-status curation-status-badge is-pending",
                                                "{curation_badge_prerequisite_pending(locale)}"
                                            }
                                        }
                                        if matches!(row.status, CurationStatus::PendingDependencies) {
                                            span { class: "curation-status curation-status-badge is-pending",
                                                "{curation_badge_second_pass_required(locale)}"
                                            }
                                        }
                                        if row.exact_mass.is_none() {
                                            span {
                                                class: "curation-status curation-status-badge is-warn",
                                                title: "{row.mass_warning.as_deref().unwrap_or(curation_mass_warning_title(locale))}",
                                                "{curation_badge_mass_missing(locale)}"
                                            }
                                        }
                                    }
                                    div { class: "curation-note", "{row.note}" }
                                }
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

fn status_class(status: &CurationStatus) -> &'static str {
    match status {
        CurationStatus::ExistingComplete => "is-ok",
        CurationStatus::ExistingNeedsUpdates => "is-warn",
        CurationStatus::NewCompound => "is-new",
        CurationStatus::PendingDependencies => "is-pending",
        CurationStatus::Error => "is-error",
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
    value
        .map(|m| format!("{m:.5}"))
        .unwrap_or_else(|| "n/a".to_string())
}
