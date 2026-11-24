// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! Browser upload entry point for `json-count-rs`.

// wasm32 Dioxus futures capturing `Blob` or `Signal` cannot satisfy `Send`.
#![allow(clippy::future_not_send)]

use dioxus::prelude::*;
use json_count_core::{ColumnResult, progress_message, scan_source_with_progress};
use upload::{Blob, BlobSource};

use crate::app_state::JsonCountCtx;

/// Points UI signals at `blob` and launches an asynchronous chunked scan.
pub(crate) fn begin_scan_from_blob(blob: Blob, file_name: String, mut ctx: JsonCountCtx) {
    ctx.file_name.set(file_name);
    ctx.busy.set(true);
    ctx.drag_active.set(false);
    ctx.status.set("Reading file...".to_string());
    ctx.results.set(Vec::new());
    spawn_scan(blob, ctx.status, ctx.results, ctx.busy);
}

fn spawn_scan(
    blob: Blob,
    mut status: Signal<String>,
    mut results: Signal<Vec<ColumnResult>>,
    mut busy: Signal<bool>,
) {
    spawn(async move {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "DOM Blob::size returns a non-negative integer stored as f64"
        )]
        let total_bytes = blob.size() as u64;
        status.set(format!("Scanning {total_bytes} bytes..."));

        match scan_source_with_progress(
            BlobSource::new(&blob),
            total_bytes,
            move |processed, total| {
                status.set(progress_message(processed, total));
            },
        )
        .await
        {
            Ok(cols) => {
                let total: u64 = cols.iter().map(|col| col.count).sum();
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
