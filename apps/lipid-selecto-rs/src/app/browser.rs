// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! WASM streaming file ingestion and cooperative chunked classification.

use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use lipid_core::{
    BlockExtractor, LipidFormat, build_analysis_from_classified, classify_blocks, example_smiles,
    lmsd_classes,
};
use stream_io::{BlobLines, ChunkSource, SliceSource};
use upload::BlobSource;
use web_sys::Blob;

use super::UploadCtx;

const MAX_GALLERY_ITEMS: usize = 96;

/// Reads `blob` in chunks via `BlobSource` and classifies parsed spectra asynchronously.
pub(super) fn start_analysis(
    blob: Blob,
    source_name: String,
    ctx: UploadCtx,
    initial_format: Option<LipidFormat>,
) {
    spawn(async move {
        run_source(BlobSource::new(&blob), source_name, ctx, initial_format).await;
    });
}

/// Loads the curated 204-lipid dataset directly from memory via [`SliceSource`].
pub(super) fn load_example_dataset(mut ctx: UploadCtx) {
    let fmt = Some(LipidFormat::Smiles);
    ctx.input_format.set(fmt);
    let text = example_smiles().join("\n");
    spawn(async move {
        run_source(
            SliceSource::new(text.as_bytes()),
            "example_lipids.smi".to_string(),
            ctx,
            fmt,
        )
        .await;
    });
}

#[expect(
    clippy::future_not_send,
    reason = "wasm32 single-threaded executor captures Dioxus Signals and TimeoutFuture"
)]
async fn run_source<S: ChunkSource>(
    source: S,
    source_name: String,
    mut ctx: UploadCtx,
    initial_format: Option<LipidFormat>,
) {
    let total_bytes = source.total_bytes();
    ctx.file_name.set(source_name);
    ctx.status.set(format!("Reading {total_bytes} bytes…"));
    ctx.busy.set(true);

    let mut reader = BlobLines::new(source, move |processed, total| {
        ctx.status.set(format!(
            "Reading file… {}/{} bytes",
            processed.min(total),
            total
        ));
    });
    let mut extractor = BlockExtractor::with_format(initial_format);
    let mut line_count = 0usize;
    loop {
        match reader.next_line().await {
            Ok(Some(line)) => {
                line_count += 1;
                extractor.push_line(&line);
            }
            Ok(None) => break,
            Err(err) => {
                ctx.status.set(format!("Error reading file: {err}"));
                ctx.busy.set(false);
                return;
            }
        }
    }

    if initial_format.is_none()
        && let Some(detected) = extractor.detected_format()
    {
        ctx.input_format.set(Some(detected));
    }
    let mut blocks = extractor.finish();
    if blocks.is_empty() {
        ctx.status
            .set("No valid entries found in the file.".to_string());
        ctx.busy.set(false);
        return;
    }

    ctx.status
        .set(format!("Parsed {line_count} lines — classifying…"));
    let all_classes = lmsd_classes();
    let total = blocks.len();
    let mut processed = 0usize;
    for chunk in blocks.chunks_mut(500) {
        classify_blocks(chunk, all_classes);
        processed += chunk.len();
        ctx.status
            .set(format!("Classifying… {processed}/{total} entries"));
        TimeoutFuture::new(0).await;
    }
    ctx.status
        .set(format!("Classified {total} entries — building gallery…"));

    let result = build_analysis_from_classified(blocks, MAX_GALLERY_ITEMS, all_classes);
    let block_count = result.blocks.len();
    let gallery_count = result.gallery.len();
    ctx.analysis.set(Some(result));
    ctx.status.set(format!(
        "Done — {block_count} entries analyzed (showing {gallery_count} gallery items)."
    ));
    ctx.busy.set(false);
    TimeoutFuture::new(0).await;
}
