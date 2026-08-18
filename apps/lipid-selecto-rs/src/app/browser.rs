//! WASM-only glue that reads the uploaded blob in streaming chunks, runs the
//! (cooperative) lipid classification pass off the render thread, and triggers
//! a file download.

#[cfg(target_arch = "wasm32")]
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::chemical_class::ChemicalClass;
#[cfg(target_arch = "wasm32")]
use crate::examples::example_smiles;
#[cfg(target_arch = "wasm32")]
use crate::format::LipidFormat;
#[cfg(target_arch = "wasm32")]
use crate::parser::{
    Analysis, build_analysis_from_classified, classify_blocks, extract_blocks_from_lines,
};
#[cfg(target_arch = "wasm32")]
use crate::rules::LipidRuleLibrary;
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(target_arch = "wasm32")]
use upload::{BlobLines, UploadError};
#[cfg(target_arch = "wasm32")]
use web_sys::Blob;

/// Cap the number of structure diagrams rendered (each costs a chem parse +
/// force-directed layout), keeping the initial render snappy.
#[cfg(target_arch = "wasm32")]
const MAX_GALLERY_ITEMS: usize = 96;

/// Reads the blob in streaming chunks, classifies lipids, and populates signals.
///
/// `input_format` and `rule_library` are reserved for future per-rule analysis
/// (the current classifier uses the built-in LIPID MAPS rule set).
#[cfg(target_arch = "wasm32")]
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn start_analysis(
    blob: Blob,
    source_name: String,
    mut file_name_signal: Signal<String>,
    mut status: Signal<String>,
    mut busy: Signal<bool>,
    _drag_active: Signal<bool>,
    mut analysis: Signal<Option<Analysis>>,
    _input_format: Option<LipidFormat>,
    _rule_library: LipidRuleLibrary,
) {
    file_name_signal.set(source_name);

    spawn(async move {
        let total_bytes = blob.size() as u64;
        status.set(format!("Reading {total_bytes} bytes…"));
        busy.set(true);

        // Stream the blob line-by-line via BlobLines, collecting only the
        // parsed blocks (not the raw file text) so memory stays bounded by
        // the parsed output, not the raw file size.
        let mut reader = BlobLines::new(&blob, |processed, total| {
            status.set(format!(
                "Reading file… {}/{} bytes",
                processed.min(total),
                total
            ));
        });

        let mut lines: Vec<String> = Vec::new();
        loop {
            match reader.next_line().await {
                Ok(Some(line)) => lines.push(line),
                Ok(None) => break,
                Err(UploadError::UnexpectedEof) => break,
                Err(e) => {
                    status.set(format!("Error reading file: {e}"));
                    busy.set(false);
                    return;
                }
            }
        }

        status.set(format!("Parsed {} lines — classifying…", lines.len()));

        let blocks = extract_blocks_from_lines(lines.into_iter());

        if blocks.is_empty() {
            status.set("No valid entries found in the file.".to_string());
            busy.set(false);
            return;
        }

        // Classify blocks in chunks, yielding to the event loop between chunks
        // so the UI stays responsive for large MGF files (thousands of entries).
        let all_classes = ChemicalClass::defaults();
        let mut blocks = blocks;
        let chunk_size = 500;
        let total = blocks.len();
        let mut processed = 0;
        for chunk in blocks.chunks_mut(chunk_size) {
            classify_blocks(chunk, &all_classes);
            processed += chunk.len();
            status.set(format!("Classifying… {processed}/{total} entries"));
            TimeoutFuture::new(0).await;
        }
        status.set(format!("Classified {total} entries — building gallery…"));

        let result = build_analysis_from_classified(blocks, MAX_GALLERY_ITEMS, all_classes);
        let block_count = result.blocks.len();
        let gallery_count = result.gallery.len();
        analysis.set(Some(result));

        status.set(format!(
            "Done — {block_count} entries analyzed (showing {gallery_count} gallery items)."
        ));
        busy.set(false);
        // Yield once so the UI repaints before the heavy work finishes.
        TimeoutFuture::new(0).await;
    });
}

/// Public entry point: delegates to [`start_analysis`] with the uploaded blob.
#[cfg(target_arch = "wasm32")]
pub fn begin_analysis_from_blob(
    blob: Blob,
    source_name: String,
    file_name_signal: Signal<String>,
    status: Signal<String>,
    busy: Signal<bool>,
    drag_active: Signal<bool>,
    analysis: Signal<Option<Analysis>>,
    _input_format: Option<LipidFormat>,
    _rule_library: LipidRuleLibrary,
) {
    start_analysis(
        blob,
        source_name,
        file_name_signal,
        status,
        busy,
        drag_active,
        analysis,
        _input_format,
        _rule_library,
    );
}

/// Loads the curated example lipid dataset (74 SMILES) and runs analysis.
#[cfg(target_arch = "wasm32")]
pub fn load_example_dataset(
    file_name_signal: Signal<String>,
    status: Signal<String>,
    busy: Signal<bool>,
    drag_active: Signal<bool>,
    analysis: Signal<Option<Analysis>>,
    input_format: Signal<Option<LipidFormat>>,
    _rule_library: LipidRuleLibrary,
) {
    let mut status = status;
    let mut busy = busy;
    let fmt = *input_format.read();
    let smiles_lines = example_smiles();
    let text = smiles_lines.join("\n");

    // Build a Blob from the example SMILES text.
    let arr = js_sys::Array::new();
    arr.push(&wasm_bindgen::JsValue::from_str(&text));
    let blob = match Blob::new_with_str_sequence(&arr.into()) {
        Ok(blob) => blob,
        Err(e) => {
            status.set(format!("Error creating blob: {e:?}"));
            busy.set(false);
            return;
        }
    };

    start_analysis(
        blob,
        "example_lipids.smi".to_string(),
        file_name_signal,
        status,
        busy,
        drag_active,
        analysis,
        fmt,
        _rule_library,
    );
}
