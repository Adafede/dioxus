//! Upload dispatch — mirrors `mgf-precursor-erro-rs`' `app/browser.rs` cfg-mut
//! pattern: the wasm arm forwards to `pipeline::begin_import` /
//! `begin_import_from_text`, the native arm pins the signals to the
//! "needs browser" state. Keeps `app.rs` free of duplicated
//! `#[cfg(target_arch = "wasm32")]` import blocks.
use dioxus::html::FileData;
use dioxus::prelude::*;

use crate::model::{EndpointStatus, MoleculeRow, MotifSummary};
#[cfg(target_arch = "wasm32")]
use crate::pipeline::{begin_import, begin_import_from_text};

/// Shared dispatch for `onchange`/`ondrop` file events: selects the first file
/// from `files` and (wasm) kicks off [`begin_import`], (native) reports that the
/// app needs a browser.
#[allow(clippy::too_many_arguments, unused_mut, unused_variables)]
pub fn attempt_import(
    files: &[FileData],
    mut file_name: Signal<String>,
    mut status: Signal<String>,
    mut busy: Signal<bool>,
    mut drag_active: Signal<bool>,
    mut rows: Signal<Vec<MoleculeRow>>,
    mut motifs: Signal<Vec<MotifSummary>>,
    mut endpoints: Signal<Vec<EndpointStatus>>,
    mut warnings: Signal<Vec<String>>,
) {
    let Some(file) = files.first() else {
        status.set("No file selected.".to_string());
        return;
    };

    let file_name_value = file.name();

    #[cfg(target_arch = "wasm32")]
    let Some(web_file) = file.inner().downcast_ref::<web_sys::File>() else {
        status.set("This file type is not supported in the browser.".to_string());
        return;
    };

    #[cfg(target_arch = "wasm32")]
    begin_import(
        web_file.clone(),
        file_name_value,
        file_name,
        status,
        busy,
        drag_active,
        rows,
        motifs,
        endpoints,
        warnings,
    );

    #[cfg(not(target_arch = "wasm32"))]
    {
        file_name.set(file_name_value);
        status.set("This app needs to run in a browser.".to_string());
    }
}

/// Shared dispatch for the "paste SMILES" path — (wasm) `begin_import_from_text`,
/// (native) reports that the app needs a browser.
#[allow(clippy::too_many_arguments, unused_mut, unused_variables)]
pub fn attempt_import_from_text(
    text: String,
    mut file_name: Signal<String>,
    mut status: Signal<String>,
    mut busy: Signal<bool>,
    mut drag_active: Signal<bool>,
    mut rows: Signal<Vec<MoleculeRow>>,
    mut motifs: Signal<Vec<MotifSummary>>,
    mut endpoints: Signal<Vec<EndpointStatus>>,
    mut warnings: Signal<Vec<String>>,
) {
    if text.is_empty() {
        status.set("Paste one SMILES per line or load a CSV.".to_string());
        return;
    }

    let file_name_value = "pasted-smiles.txt".to_string();

    #[cfg(target_arch = "wasm32")]
    begin_import_from_text(
        text,
        file_name_value,
        file_name,
        status,
        busy,
        drag_active,
        rows,
        motifs,
        endpoints,
        warnings,
    );

    #[cfg(not(target_arch = "wasm32"))]
    {
        status.set("This app needs to run in a browser.".to_string());
    }
}
