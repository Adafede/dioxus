// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Wasm32 async import orchestrator delegating pure Pass-1/Pass-2 logic to
//! `smellfish_core` while guarding signals against concurrent import races.

use crate::app::browser::ImportSignals;
use crate::rdkit::{inspect_smiles, read_file_text};
use dioxus::prelude::{ReadableExt, Signal, WritableExt, spawn};
use lotus_search::reqwest_client::ReqwestClient;
use smellfish_core::csv::parse_csv_rows;
use smellfish_core::model::{EndpointStatus, Enrichment, EnrichmentOutcome};
use smellfish_core::pipeline::{ImportOutcome, RowInspection, finalize_pass2, prepare_pass1};
use smellfish_core::qlever::enrich_sources;

pub(crate) enum ImportSource {
    Blob(web_sys::Blob),
    Text(String),
}

pub(crate) fn begin_import(
    source: ImportSource,
    file_name_value: String,
    mut signals: ImportSignals,
) {
    let expected_gen = {
        let next = *signals.generation.peek() + 1;
        signals.generation.set(next);
        next
    };

    signals.file_name.set(file_name_value);
    signals.busy.set(true);
    signals.drag_active.set(false);
    signals.status.set("Preparing…".to_string());
    signals.rows.set(Vec::new());
    signals.motifs.set(Vec::new());
    signals.endpoints.set(Vec::new());
    signals.warnings.set(Vec::new());

    spawn(async move {
        let outcome = run_import(source, signals.generation, expected_gen, signals.status).await;
        if *signals.generation.peek() != expected_gen {
            return;
        }
        match outcome {
            Ok(outcome) => apply_import_outcome(outcome, "Done", signals),
            Err(err) => signals.status.set(format!("Error analyzing input: {err}")),
        }
        signals.busy.set(false);
    });
}

#[expect(
    clippy::future_not_send,
    reason = "wasm32 JS Blob/bridge futures are single-threaded (!Send)"
)]
async fn run_import(
    source: ImportSource,
    generation: Signal<u64>,
    expected_gen: u64,
    status: Signal<String>,
) -> Result<ImportOutcome, String> {
    let text = match source {
        ImportSource::Blob(file) => {
            set_status_if_current(status, generation, expected_gen, "Reading CSV file…");
            read_file_text(&file).await?
        }
        ImportSource::Text(text) => text,
    };

    set_status_if_current(status, generation, expected_gen, "Parsing CSV…");
    let raw_rows = parse_csv_rows(&text).map_err(|err| err.to_string())?;
    let total = raw_rows.len();

    set_status_if_current(status, generation, expected_gen, "Loading RDKit module…");
    let mut inspections = Vec::with_capacity(total);
    for (idx, raw) in raw_rows.into_iter().enumerate() {
        if *generation.peek() != expected_gen {
            return Err("cancelled".to_string());
        }
        let current = idx + 1;
        set_status_if_current(
            status,
            generation,
            expected_gen,
            &format!("Inspecting structure {current}/{total}…"),
        );
        let outcome = inspect_smiles(&raw.smiles).await;
        if *generation.peek() != expected_gen {
            return Err("cancelled".to_string());
        }
        inspections.push(RowInspection { raw, outcome });
    }

    if *generation.peek() != expected_gen {
        return Err("cancelled".to_string());
    }
    let batch = prepare_pass1(inspections);
    set_status_if_current(
        status,
        generation,
        expected_gen,
        "Probing LOTUS and PubChem…",
    );

    let enrichment = match ReqwestClient::new() {
        Ok(http) => {
            enrich_sources(&http, &batch.unique_inchikeys, |msg| {
                set_status_if_current(status, generation, expected_gen, msg);
            })
            .await
        }
        Err(err) => EnrichmentOutcome {
            enrichment: Enrichment::default(),
            endpoints: vec![EndpointStatus {
                name: "HTTP client".to_string(),
                endpoint: String::new(),
                reachable: false,
                detail: err.to_string(),
            }],
            warnings: vec![format!("could not open an HTTP client: {err}")],
        },
    };

    if *generation.peek() != expected_gen {
        return Err("cancelled".to_string());
    }
    set_status_if_current(status, generation, expected_gen, "Assessing NP evidence…");
    Ok(finalize_pass2(batch, enrichment))
}

fn set_status_if_current(
    mut status: Signal<String>,
    generation: Signal<u64>,
    expected_gen: u64,
    message: &str,
) {
    if *generation.peek() == expected_gen {
        status.set(message.to_string());
    }
}

fn apply_import_outcome(outcome: ImportOutcome, prefix: &str, mut signals: ImportSignals) {
    let row_count = outcome.rows.len();
    let motif_count = outcome.motifs.len();
    let sub_count = outcome.unique_substituents;
    let unique = outcome.unique_inchikeys;
    let has_warnings = !outcome.warnings.is_empty();
    signals.rows.set(outcome.rows);
    signals.motifs.set(outcome.motifs);
    signals.endpoints.set(outcome.endpoints);
    signals.warnings.set(outcome.warnings);
    let suffix = if has_warnings {
        " with QLever warnings"
    } else {
        ""
    };
    signals.status.set(format!(
        "{prefix}{suffix} — {row_count} results, {motif_count} motifs, {sub_count} Ertl substituents, {unique} unique InChIKeys"
    ));
}
