// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! Browser upload entry point for `json-count-rs`.

use dioxus::prelude::*;
use json_count_core::{ColumnResult, progress_message, scan_source_with_progress};
use upload::{Blob, BlobSource, UploadGen};

use crate::app_state::JsonCountCtx;

/// Points UI signals at `blob` and launches an asynchronous chunked scan.
pub(crate) fn begin_scan_from_blob(blob: Blob, file_name: String, mut ctx: JsonCountCtx) {
    let next_gen = ctx.generation.advance();
    ctx.file_name.set(file_name);
    ctx.busy.set(true);
    ctx.drag_active.set(false);
    ctx.status.set("Reading file...".to_string());
    ctx.results.set(Vec::new());
    spawn_scan(
        blob,
        next_gen,
        ctx.generation,
        ctx.status,
        ctx.results,
        ctx.busy,
    );
}

fn spawn_scan(
    blob: Blob,
    expected_gen: u64,
    generation: UploadGen,
    mut status: Signal<String>,
    mut results: Signal<Vec<ColumnResult>>,
    mut busy: Signal<bool>,
) {
    spawn(async move {
        let source = BlobSource::new(&blob);
        status.set(format!("Scanning {} bytes...", source.total_bytes()));

        let outcome = scan_source_with_progress(source, move |processed, total| {
            if generation.is_current(expected_gen) {
                status.set(progress_message(processed, total));
            }
        })
        .await;

        if !generation.is_current(expected_gen) {
            return;
        }

        match outcome {
            Ok(cols) => {
                let total: u64 = cols
                    .iter()
                    .map(|col| col.count)
                    .fold(0u64, u64::saturating_add);
                status.set(format!(
                    "Done — {} columns, {} total non-null values",
                    cols.len(),
                    total
                ));
                results.set(cols);
            }
            Err(error) => {
                status.set(format!("Error reading file: {error}"));
                results.set(Vec::new());
            }
        }
        busy.set(false);
    });
}
