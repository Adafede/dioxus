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
use ui::prelude::*;

const NA_TEXT: &str = "n/a";

#[component]
fn StatusSummaryBadges(locale: Locale, rows: Arc<[CurationResultRow]>) -> Element {
    rsx! {
        div { style: status_badges_style(),
            for (status, count) in status_counts(rows.as_ref()) {
                span { style: status_pill_style(&status),
                    "{status_label(locale, &status)} ({count})"
                }
            }
        }
    }
}

fn render_curation_result_cells(locale: Locale, row: &CurationResultRow) -> Element {
    rsx! {
        td { style: curation_status_cell_style(),
            span { style: status_pill_style(&row.status),
                "{status_label(locale, &row.status)}"
            }
            div { style: row_badges_style(),
                if !row.dependency_blocks.is_empty() {
                    span { style: status_pill_style(&CurationStatus::PendingDependencies),
                        "{curation_badge_prerequisite_pending(locale)}"
                    }
                }
                if matches!(row.status, CurationStatus::PendingDependencies) {
                    span { style: status_pill_style(&CurationStatus::PendingDependencies),
                        "{curation_badge_second_pass_required(locale)}"
                    }
                }
                if row.exact_mass.is_none() {
                    span {
                        style: status_warning_pill_style(),
                        title: "{row.mass_warning.as_deref().unwrap_or(curation_mass_warning_title(locale))}",
                        "{curation_badge_mass_missing(locale)}"
                    }
                }
            }
            div { style: curation_note_style(), "{row.note}" }
        }
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
        td { "{row.input.name}" }
        td { class: "mono", style: curation_cell_wrap_style(), "{row.input.smiles}" }
        td { class: "mono", style: curation_cell_wrap_style(),
            "{row.canonical_smiles.as_deref().unwrap_or(NA_TEXT)}"
        }
        td { class: "mono", "{row.inchikey.as_deref().unwrap_or(NA_TEXT)}" }
        td { class: "mono", style: curation_cell_wrap_style(),
            "{row.inchi.as_deref().unwrap_or(NA_TEXT)}"
        }
        td { class: "mono", "{row.formula.as_deref().unwrap_or(NA_TEXT)}" }
        td { class: "mono", "{format_mass(row.exact_mass)}" }
    }
}

#[component]
pub fn CurationResultsTable(locale: Locale, rows: Arc<[CurationResultRow]>) -> Element {
    let scroll_hint_id = "curation-results-scroll-hint";

    rsx! {
        div { style: curation_card_style(),
            h3 { "{crate::i18n::heading_results(locale)}" }
            StatusSummaryBadges { locale, rows: rows.clone() }
            p {
                id: scroll_hint_id,
                style: curation_scroll_hint_style(),
                span { style: scroll_hint_icon_style(), "↔" }
                "{hint_scroll_curation_results(locale)}"
            }
            div {
                class: "curation-table-scroll",
                style: curation_table_scroll_style(),
                role: "region",
                tabindex: "0",
                aria_label: "{crate::i18n::heading_results(locale)}",
                aria_describedby: scroll_hint_id,
                table { class: "curation-table curation-results-table",
                    style: results_table_style(),
                    thead {
                        tr {
                            th { scope: "col", style: status_column_style(), "{col_status(locale)}" }
                            th { scope: "col", style: wikidata_column_style(), "Wikidata" }
                            th { scope: "col", style: name_column_style(), "{col_name(locale)}" }
                            th { scope: "col", style: smiles_column_style(), "{col_original_smiles(locale)}" }
                            th { scope: "col", style: smiles_column_style(), "{col_canonical_smiles(locale)}" }
                            th { scope: "col", style: inchikey_column_style(), "InChIKey" }
                            th { scope: "col", style: smiles_column_style(), "InChI" }
                            th { scope: "col", style: formula_column_style(), "{t(locale, TextKey::Formula)}" }
                            th { scope: "col", style: formula_column_style(), "{col_exact_mass(locale)}" }
                        }
                    }
                    tbody {
                        for (idx, row) in rows.iter().enumerate() {
                            tr { style: row_stripe_style(idx),
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

fn curation_card_style() -> String {
    StyleBuilder::new()
        .property("display", "flex")
        .property("flex-direction", "column")
        .property("gap", "10px")
        .property("padding", "12px")
        .property("border", "1px solid var(--panel-border)")
        .property("border-radius", "var(--radius)")
        .property("background", "transparent")
        .property("box-shadow", "var(--panel-shadow)")
        .build()
}

fn status_badges_style() -> String {
    StyleBuilder::new()
        .property("display", "flex")
        .property("flex-wrap", "wrap")
        .property("gap", "6px")
        .build()
}

fn row_badges_style() -> String {
    StyleBuilder::new()
        .property("display", "flex")
        .property("flex-wrap", "wrap")
        .property("gap", "4px")
        .property("margin-top", "4px")
        .build()
}

fn curation_status_cell_style() -> String {
    StyleBuilder::new()
        .property("font-weight", "700")
        .property("color", "var(--text)")
        .build()
}

fn curation_note_style() -> String {
    StyleBuilder::new()
        .property("font-size", "var(--fs-label)")
        .property("color", "var(--text)")
        .property("margin-top", "3px")
        .property("white-space", "pre-line")
        .build()
}

fn curation_cell_wrap_style() -> String {
    StyleBuilder::new()
        .property("white-space", "pre-wrap")
        .property("overflow-wrap", "anywhere")
        .property("word-break", "break-word")
        .build()
}

fn curation_scroll_hint_style() -> String {
    StyleBuilder::new()
        .property("display", "inline-flex")
        .property("align-items", "center")
        .property("gap", "8px")
        .property("color", "var(--text3)")
        .property("font-size", "var(--fs-0)")
        .property("line-height", "1.4")
        .build()
}

fn scroll_hint_icon_style() -> String {
    StyleBuilder::new()
        .property("color", "var(--accent)")
        .property("font-weight", "700")
        .property("font-size", "1.05em")
        .build()
}

fn curation_table_scroll_style() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("min-width", "0")
        .property("overflow-x", "auto")
        .property("overflow-y", "visible")
        .property("border", "1px solid var(--panel-border)")
        .property("background", "var(--panel-bg-soft)")
        .property("box-shadow", "var(--panel-shadow)")
        .property(
            "transition",
            "background .15s ease, border-color .15s ease, box-shadow .15s ease",
        )
        .build()
}

fn results_table_style() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("border-collapse", "collapse")
        .property("font-size", "var(--fs-ui)")
        .property("table-layout", "auto")
        .property("word-break", "break-word")
        .build()
}

fn status_pill_style(status: &CurationStatus) -> String {
    let border_color = match status {
        CurationStatus::ExistingComplete => "var(--wd-taxon)",
        CurationStatus::ExistingNeedsUpdates => "var(--footer-wd-entries)",
        CurationStatus::NewCompound => "var(--wd-reference)",
        CurationStatus::PendingDependencies => "var(--wd-reference)",
        CurationStatus::Error => "var(--wd-compound)",
    };

    StyleBuilder::new()
        .property("display", "inline-flex")
        .property("align-items", "center")
        .property("padding", "2px 8px")
        .property("border-radius", "4px")
        .property("border-left", "3px solid transparent")
        .property("border-left-color", border_color)
        .property(
            "background",
            "color-mix(in srgb, var(--surface) 90%, transparent)",
        )
        .property("font-weight", "700")
        .property("text-transform", "uppercase")
        .property("font-size", "var(--fs-micro)")
        .property("letter-spacing", "0.04em")
        .property("color", "var(--text)")
        .build()
}

fn status_warning_pill_style() -> String {
    StyleBuilder::new()
        .property("display", "inline-flex")
        .property("align-items", "center")
        .property("padding", "2px 8px")
        .property("border-radius", "4px")
        .property("border-left", "3px solid transparent")
        .property("border-left-color", "var(--footer-wd-entries)")
        .property(
            "background",
            "color-mix(in srgb, var(--surface) 90%, transparent)",
        )
        .property("font-weight", "700")
        .property("text-transform", "uppercase")
        .property("font-size", "var(--fs-micro)")
        .property("letter-spacing", "0.04em")
        .property("color", "var(--text)")
        .build()
}

fn status_column_style() -> String {
    StyleBuilder::new().property("min-width", "220px").build()
}

fn wikidata_column_style() -> String {
    StyleBuilder::new().property("min-width", "12ch").build()
}

fn name_column_style() -> String {
    StyleBuilder::new().property("min-width", "18ch").build()
}

fn smiles_column_style() -> String {
    StyleBuilder::new()
        .property("min-width", "220px")
        .property("max-width", "320px")
        .build()
}

fn inchikey_column_style() -> String {
    StyleBuilder::new().property("min-width", "28ch").build()
}

fn formula_column_style() -> String {
    StyleBuilder::new().property("min-width", "12ch").build()
}

fn row_stripe_style(idx: usize) -> String {
    let background = if idx.is_multiple_of(2) {
        "color-mix(in srgb, var(--surface) 94%, transparent)"
    } else {
        "color-mix(in srgb, var(--surface) 88%, transparent)"
    };

    StyleBuilder::new()
        .property("transition", "background .14s ease")
        .property("--row-bg", background)
        .build()
}
