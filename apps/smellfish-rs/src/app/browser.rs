// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Upload dispatch — mirrors `mgf-precursor-erro-rs`' `app/browser.rs` cfg-mut
//! pattern: the wasm arm forwards to `pipeline::begin_import`, the native arm
//! pins the signals to the "needs browser" state. Keeps `app.rs` free of
//! duplicated `#[cfg(target_arch = "wasm32")]` import blocks.
use dioxus::html::FileData;
use dioxus::prelude::*;

use crate::model::{EndpointStatus, MoleculeRow, MotifSummary};
#[cfg(target_arch = "wasm32")]
use crate::pipeline::{ImportSource, begin_import};

#[derive(Clone, Copy)]
pub(crate) struct ImportSignals {
    pub(crate) file_name: Signal<String>,
    pub(crate) status: Signal<String>,
    pub(crate) busy: Signal<bool>,
    pub(crate) drag_active: Signal<bool>,
    pub(crate) rows: Signal<Vec<MoleculeRow>>,
    pub(crate) motifs: Signal<Vec<MotifSummary>>,
    pub(crate) endpoints: Signal<Vec<EndpointStatus>>,
    pub(crate) warnings: Signal<Vec<String>>,
}

/// Shared dispatch for `onchange`/`ondrop` file events: selects the first file
/// from `files` and (wasm) kicks off `pipeline::begin_import`, (native) reports that the
/// app needs a browser.
pub(super) fn attempt_import(files: &[FileData], mut signals: ImportSignals) {
    match upload::extract_blob_from_file_data(files) {
        Ok(Some(file)) => {
            #[cfg(target_arch = "wasm32")]
            begin_import(ImportSource::Blob(file.blob), file.name, signals);

            #[cfg(not(target_arch = "wasm32"))]
            {
                signals.file_name.set(file.name);
                signals
                    .status
                    .set("This app needs to run in a browser.".to_string());
            }
        }
        Ok(None) => signals.status.set("No file selected.".to_string()),
        Err(msg) => signals.status.set(msg),
    }
}

/// Shared dispatch for the "paste SMILES" path — (wasm) `pipeline::begin_import`,
/// (native) reports that the app needs a browser.
pub(super) fn attempt_import_from_text(text: &str, mut signals: ImportSignals) {
    if text.is_empty() {
        signals
            .status
            .set("Paste one SMILES per line or load a CSV.".to_string());
        return;
    }

    #[cfg(target_arch = "wasm32")]
    begin_import(
        ImportSource::Text(text.to_string()),
        "pasted-smiles.txt".to_string(),
        signals,
    );

    #[cfg(not(target_arch = "wasm32"))]
    {
        signals
            .status
            .set("This app needs to run in a browser.".to_string());
    }
}
