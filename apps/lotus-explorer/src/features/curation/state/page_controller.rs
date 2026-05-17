// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::curation::{CurationInputRow, CurationResultRow, QuickStatementsBundle, parse_tsv_rows};
use crate::features::curation::queue::append_unique_rows;
use crate::features::curation::workflow;
use crate::i18n::{Locale, msg_no_valid_tsv_rows, msg_running_checks, msg_tsv_import_complete};
use dioxus::prelude::*;
use std::sync::Arc;

pub fn start_curation_run(
    locale: Locale,
    snapshot: Vec<CurationInputRow>,
    mut processing: Signal<bool>,
    mut status_message: Signal<Option<String>>,
    mut result_rows: Signal<Arc<[CurationResultRow]>>,
    mut quickstatements: Signal<QuickStatementsBundle>,
    mut awaiting_second_pass: Signal<bool>,
) {
    processing.set(true);
    status_message.set(Some(msg_running_checks(locale)));
    spawn(async move {
        match workflow::run_curation(locale, snapshot).await {
            Ok(outcome) => {
                awaiting_second_pass.set(outcome.awaiting_second_pass);
                result_rows.set(outcome.result_rows);
                quickstatements.set(outcome.quickstatements);
                processing.set(false);
                status_message.set(Some(outcome.status_message));
            }
            Err(err) => {
                processing.set(false);
                status_message.set(Some(workflow::format_curation_error_typed(locale, &err)));
            }
        }
    });
}

pub fn import_tsv_rows(
    locale: Locale,
    content: &str,
    mut rows: Signal<Vec<CurationInputRow>>,
    mut status_message: Signal<Option<String>>,
) {
    match parse_tsv_rows(content) {
        Ok(parsed) => {
            if parsed.is_empty() {
                status_message.set(Some(msg_no_valid_tsv_rows(locale)));
            } else {
                let outcome = append_unique_rows(&mut rows.write(), parsed);
                status_message.set(Some(msg_tsv_import_complete(
                    locale,
                    outcome.added,
                    outcome.skipped,
                )));
            }
        }
        Err(err) => {
            status_message.set(Some(err.to_string()));
        }
    }
}

pub fn rows_to_tsv(rows: &[CurationInputRow]) -> String {
    let mut out = String::from("name\tsmiles\ttaxon\tdoi\n");
    for row in rows {
        out.push_str(&clean_tsv_cell(&row.name));
        out.push('\t');
        out.push_str(&clean_tsv_cell(&row.smiles));
        out.push('\t');
        out.push_str(&clean_tsv_cell(row.taxon.as_deref().unwrap_or("")));
        out.push('\t');
        out.push_str(&clean_tsv_cell(row.doi.as_deref().unwrap_or("")));
        out.push('\n');
    }
    out
}

pub fn clean_tsv_cell(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], " ").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_tsv_cell_normalizes_multiline_and_tabs() {
        assert_eq!(clean_tsv_cell("  A\tB\nC\r "), "A B C");
    }

    #[test]
    fn rows_to_tsv_writes_expected_header_and_cells() {
        let rows = vec![CurationInputRow {
            name: "A\nname".to_string(),
            smiles: "CCO".to_string(),
            taxon: Some("Tax\ton".to_string()),
            doi: Some("10.1/ABC".to_string()),
        }];

        let tsv = rows_to_tsv(&rows);
        assert!(tsv.starts_with("name\tsmiles\ttaxon\tdoi\n"));
        assert!(tsv.contains("A name\tCCO\tTax on\t10.1/ABC\n"));
    }
}
