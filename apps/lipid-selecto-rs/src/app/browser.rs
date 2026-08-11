//! WASM-only glue that reads the uploaded blob, runs the (cooperative) lipid
//! classification pass off the render thread, and triggers a file download.

use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::chemical_class::ChemicalClass;
#[cfg(target_arch = "wasm32")]
use crate::parser::{Analysis, build_filtered_mgf, extract_blocks, gallery_item, summarize};
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(target_arch = "wasm32")]
use js_sys::Array;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::{Blob, HtmlAnchorElement, Url};

/// Cap the number of structure diagrams rendered (each costs a chem parse +
/// force-directed layout), keeping the initial render snappy.
#[cfg(target_arch = "wasm32")]
const MAX_GALLERY_ITEMS: usize = 96;

#[cfg(target_arch = "wasm32")]
fn start_analysis(
    blob: Blob,
    source_name: String,
    mut file_name_signal: Signal<String>,
    mut status: Signal<String>,
    mut busy: Signal<bool>,
    _drag_active: Signal<bool>,
    mut analysis: Signal<Option<Analysis>>,
    input_format: Option<crate::format::LipidFormat>,
    rule_library: crate::rules::LipidRuleLibrary,
) {
    file_name_signal.set(source_name);

    spawn(async move {
        let total_bytes = blob.size();
        status.set(format!("Reading {total_bytes} bytes…"));

        let text = match JsFuture::from(blob.text()).await {
            Ok(value) => value.as_string().unwrap_or_default(),
            Err(_) => {
                status.set("Could not read file content.".to_string());
                busy.set(false);
                return;
            }
        };
        status.set(format!(
            "Loaded {} characters — extracting items…",
            text.len()
        ));

        let mut blocks = crate::parser::extract_blocks(&text);
        let total = blocks.len();

        // Cooperative classification: yield to the browser every so often so the
        // page stays responsive on large MGF files.
        for (index, block) in blocks.iter_mut().enumerate() {
            block.classify();
            if index % 256 == 0 {
                status.set(format!("Classifying lipids… {index}/{total} items"));
                TimeoutFuture::new(0).await;
            }
        }

        // Now that we have classification, compute class matches for all blocks
        let all_classes = crate::chemical_class::ChemicalClass::defaults();
        for block in &mut blocks {
            block.compute_class_matches(&all_classes);
        }

        let lipid_count = blocks.iter().filter(|block| block.is_lipid()).count();
        status.set(format!("Rendering {lipid_count} structures…"));

        let mut gallery = Vec::with_capacity(lipid_count.min(MAX_GALLERY_ITEMS));
        for block in blocks.iter().filter(|block| block.is_lipid()) {
            if gallery.len() >= MAX_GALLERY_ITEMS {
                break;
            }
            gallery.push(crate::parser::gallery_item(block, &all_classes));
            if gallery.len() % 16 == 0 {
                status.set(format!(
                    "Rendering structures… {}/{}",
                    gallery.len(),
                    lipid_count.min(MAX_GALLERY_ITEMS)
                ));
                TimeoutFuture::new(0).await;
            }
        }

        let summary = crate::parser::summarize(&blocks);
        let filtered_mgf = crate::parser::build_filtered_mgf(&blocks);

        analysis.set(Some(Analysis {
            summary,
            gallery,
            filtered_mgf,
            blocks,
            all_classes,
        }));

        status.set(format!(
            "Selected {lipid_count} lipid items out of {total} — ready for analysis.",
        ));
        busy.set(false);
    });
}

#[cfg(target_arch = "wasm32")]
pub fn begin_analysis_from_blob(
    blob: Blob,
    file_name: String,
    mut file_name_signal: Signal<String>,
    mut status: Signal<String>,
    mut busy: Signal<bool>,
    mut drag_active: Signal<bool>,
    analysis: Signal<Option<Analysis>>,
    input_format: Option<crate::format::LipidFormat>,
    rule_library: crate::rules::LipidRuleLibrary,
) {
    file_name_signal.set(file_name.clone());
    busy.set(true);
    drag_active.set(false);
    status.set("Starting analysis…".to_string());

    start_analysis(
        blob,
        file_name,
        file_name_signal,
        status,
        busy,
        drag_active,
        analysis,
        input_format,
        rule_library,
    );
}

/// Load the 100-example SMILES dataset as if it were uploaded.
#[cfg(target_arch = "wasm32")]
pub fn load_example_dataset(
    mut file_name_signal: dioxus::prelude::Signal<String>,
    mut status: dioxus::prelude::Signal<String>,
    mut busy: dioxus::prelude::Signal<bool>,
    mut drag_active: dioxus::prelude::Signal<bool>,
    mut analysis: dioxus::prelude::Signal<Option<crate::parser::Analysis>>,
    mut input_format: dioxus::prelude::Signal<Option<crate::format::LipidFormat>>,
    rule_library: crate::rules::LipidRuleLibrary,
) -> Result<(), String> {
    use crate::examples::example_smiles;
    use web_sys::Blob;

    let examples = example_smiles();
    let content = examples.join("\n");

    let array = Array::new();
    array.push(&wasm_bindgen::JsValue::from(&content));
    let blob = Blob::new_with_str_sequence(&array)
        .map_err(|_| "Failed to create blob from examples.".to_string())?;

    drag_active.set(false);
    busy.set(true);
    start_analysis(
        blob,
        "example_lipids.smi".to_string(),
        file_name_signal,
        status,
        busy,
        drag_active,
        analysis,
        Some(crate::format::LipidFormat::Smiles),
        rule_library,
    );
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_example_dataset(
    _file_name_signal: dioxus::prelude::Signal<String>,
    _status: dioxus::prelude::Signal<String>,
    _busy: dioxus::prelude::Signal<bool>,
    _drag_active: dioxus::prelude::Signal<bool>,
    _analysis: dioxus::prelude::Signal<Option<crate::parser::Analysis>>,
    _input_format: dioxus::prelude::Signal<Option<crate::format::LipidFormat>>,
    _rule_library: crate::rules::LipidRuleLibrary,
) -> Result<(), String> {
    Err("Example loading is only available in the browser (use `dx serve`).".to_string())
}
