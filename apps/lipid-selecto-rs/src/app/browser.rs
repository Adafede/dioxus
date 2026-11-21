// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! WASM-only glue that reads the uploaded blob in streaming chunks, runs the
//! (cooperative) lipid classification pass off the render thread, and triggers
//! a file download.

use dioxus::prelude::*;

use super::UploadCtx;
use crate::examples::example_smiles;
use crate::format::LipidFormat;
use crate::parser::{BlockExtractor, build_analysis_from_classified, classify_blocks};
use gloo_timers::future::TimeoutFuture;
use upload::{BlobLines, BlobSource, UploadError};
use web_sys::Blob;

/// Cap the number of structure diagrams rendered (each costs a chem parse +
/// force-directed layout), keeping the initial render snappy.
const MAX_GALLERY_ITEMS: usize = 96;

/// Reads the blob in streaming chunks, classifies lipids, and populates signals.
#[allow(clippy::too_many_lines)] // wasm async dispatch needs streaming loop and chunked yield
pub(super) fn start_analysis(
    blob: Blob,
    source_name: String,
    mut ctx: UploadCtx,
    initial_format: Option<LipidFormat>,
) {
    ctx.file_name.set(source_name);

    spawn(async move {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // blob.slice_with_f64_and_f64 requires f64 args, cast from u64
        let total_bytes = blob.size() as u64;
        ctx.status.set(format!("Reading {total_bytes} bytes…"));
        ctx.busy.set(true);

        // Stream the blob line-by-line via BlobLines, collecting only the
        // parsed blocks (not the raw file text) so memory stays bounded by
        // the parsed output, not the raw file size.
        let mut reader = BlobLines::new(BlobSource::new(&blob), |processed, total| {
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
                Ok(None) | Err(UploadError::UnexpectedEof) => break,
                Err(e) => {
                    ctx.status.set(format!("Error reading file: {e}"));
                    ctx.busy.set(false);
                    return;
                }
            }
        }

        ctx.status
            .set(format!("Parsed {line_count} lines — classifying…"));

        if initial_format.is_none()
            && let Some(detected) = extractor.detected_format()
        {
            ctx.input_format.set(Some(detected));
        }
        let blocks = extractor.finish();

        if blocks.is_empty() {
            ctx.status
                .set("No valid entries found in the file.".to_string());
            ctx.busy.set(false);
            return;
        }

        // Classify blocks in chunks, yielding to the event loop between chunks
        // so the UI stays responsive for large MGF files (thousands of entries).
        let all_classes = crate::chemical_class::lmsd_all();
        let mut blocks = blocks;
        let chunk_size = 500;
        let total = blocks.len();
        let mut processed = 0;
        for chunk in blocks.chunks_mut(chunk_size) {
            classify_blocks(chunk, &all_classes);
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
        // Yield once so the UI repaints before the heavy work finishes.
        TimeoutFuture::new(0).await;
    });
}

/// Loads the curated example lipid dataset (204 SMILES) and runs analysis.
pub(super) fn load_example_dataset(mut ctx: UploadCtx) {
    let fmt = Some(LipidFormat::Smiles);
    ctx.input_format.set(fmt);
    let smiles_lines = example_smiles();
    let text = smiles_lines.join("\n");

    // Build a Blob from the example SMILES text.
    let arr = js_sys::Array::new();
    arr.push(&wasm_bindgen::JsValue::from_str(&text));
    let blob = match Blob::new_with_str_sequence(&arr.into()) {
        Ok(blob) => blob,
        Err(e) => {
            ctx.status.set(format!("Error creating blob: {e:?}"));
            ctx.busy.set(false);
            return;
        }
    };

    start_analysis(blob, "example_lipids.smi".to_string(), ctx, fmt);
}
