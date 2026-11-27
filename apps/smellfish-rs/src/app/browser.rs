// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Upload dispatch and generation tracking preventing concurrent import races.

use dioxus::html::FileData;
use dioxus::prelude::*;
use smellfish_core::model::{EndpointStatus, MoleculeRow, MotifSummary};

#[cfg(target_arch = "wasm32")]
use crate::pipeline::{ImportSource, begin_import};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ImportSignals {
    pub(crate) generation: upload::UploadGen,
    pub(crate) file_name: Signal<String>,
    pub(crate) status: Signal<String>,
    pub(crate) busy: Signal<bool>,
    pub(crate) drag_active: Signal<bool>,
    pub(crate) rows: Signal<Vec<MoleculeRow>>,
    pub(crate) motifs: Signal<Vec<MotifSummary>>,
    pub(crate) endpoints: Signal<Vec<EndpointStatus>>,
    pub(crate) warnings: Signal<Vec<String>>,
}

/// Dispatches a file-drop or browse event into the import pipeline (wasm) or sets browser hint (host).
pub(super) fn attempt_import(files: &[FileData], signals: &ImportSignals) {
    let signals = *signals;
    upload::dispatch_file_upload(files, signals.status, |file| {
        #[cfg(target_arch = "wasm32")]
        begin_import(ImportSource::Blob(file.blob), file.name, signals);

        #[cfg(not(target_arch = "wasm32"))]
        upload::report_browser_only_upload(signals.file_name, signals.status, file.name);
    });
}

/// Dispatches pasted SMILES lines into the import pipeline (wasm) or sets browser hint (host).
pub(super) fn attempt_import_from_text(text: &str, signals: &ImportSignals) {
    let mut signals = *signals;
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
