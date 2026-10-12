// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! The browser entry points: everything here takes a `Blob` and Dioxus signals.
//!
//! A file of its own, and the reason is in `.cargo/mutants.toml`.
//!
//! `begin_scan_from_blob` takes a browser `Blob` and seven Dioxus `Signal`s, and
//! it is reached from a `drop` handler in the component. `dioxus` is a wasm-only
//! dependency of this crate, so a host test run — the only kind of run `cargo
//! mutants` does — cannot link it at all, let alone construct a `Blob`. Every
//! mutant in a function like that survives: not because the behaviour is
//! untested in principle, but because nothing can reach it.
//!
//! That was the reason eight mutants in `processing.rs` survived a run this file
//! used to be part of, and they were eight more than necessary: three of them
//! were the arithmetic that builds the progress line, which has nothing to do
//! with the browser and now lives in `progress_message` with tests. The five
//! that remain are the parts that genuinely are the browser — `blob.size()`,
//! `BlobSource::new`, and the three `spawn`s — and they are here rather than
//! excluded in place so that what is not covered is a name in one file.
//!
//! The alternative was to take `dioxus` out of the host build as a
//! dev-dependency and test the signals too. That is a much larger change to this
//! crate's dependency graph, and it would test the dioxus runtime rather than
//! this grammar, so it is not what the eight mutants were about.
#![allow(clippy::future_not_send)] // wasm async functions capture non-Send browser JS futures

use dioxus::prelude::*;
use upload::{Blob, BlobSource};

use super::{ColumnResult, progress_message, scan_source_with_progress};

/// Point the signals at `blob` and start a scan of it.
pub(crate) fn begin_scan_from_blob(
    blob: Blob,
    file_name: String,
    mut file_name_signal: Signal<String>,
    mut status: Signal<String>,
    mut results: Signal<Vec<ColumnResult>>,
    mut busy: Signal<bool>,
    mut drag_active: Signal<bool>,
) {
    file_name_signal.set(file_name);
    busy.set(true);
    drag_active.set(false);
    status.set("Reading file...".to_string());
    results.set(vec![]);
    spawn_scan(blob, status, results, busy);
}

/// Run the scan off the render pass, so a multi-gigabyte file does not block the
/// browser's main thread while it is read.
fn spawn_scan(
    blob: Blob,
    mut status: Signal<String>,
    mut results: Signal<Vec<ColumnResult>>,
    mut busy: Signal<bool>,
) {
    spawn(async move {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // browser Blob.size() returns f64, cast to u64 for byte counts
        let total_bytes = blob.size() as u64;
        status.set(format!("Scanning {total_bytes} bytes..."));

        let cols = match scan_source_with_progress(
            BlobSource::new(&blob),
            total_bytes,
            move |processed, total| {
                status.set(progress_message(processed, total));
            },
        )
        .await
        {
            Ok(cols) => cols,
            Err(error) => {
                status.set(format!("Error reading file: {error:?}"));
                Vec::new()
            }
        };

        let total: u64 = cols.iter().map(|col| col.count).sum();
        status.set(format!(
            "Done — {} columns, {} total non-null values",
            cols.len(),
            total
        ));
        results.set(cols);
        busy.set(false);
    });
}
