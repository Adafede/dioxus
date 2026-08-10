//! WASM-only glue that reads the uploaded blob, runs the (cooperative) lipid
//! classification pass off the render thread, and triggers a file download.

use dioxus::prelude::*;

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
            "Loaded {} characters — extracting spectra…",
            text.len()
        ));

        let mut blocks = extract_blocks(&text);
        let total = blocks.len();

        // Cooperative classification: yield to the browser every so often so the
        // page stays responsive on large MGF files.
        for (index, block) in blocks.iter_mut().enumerate() {
            block.classify();
            if index % 256 == 0 {
                status.set(format!("Classifying lipids… {index}/{total} spectra"));
                TimeoutFuture::new(0).await;
            }
        }

        let lipid_count = blocks.iter().filter(|block| block.is_lipid()).count();
        status.set(format!("Rendering {lipid_count} structures…"));

        let mut gallery = Vec::with_capacity(lipid_count.min(MAX_GALLERY_ITEMS));
        for block in blocks.iter().filter(|block| block.is_lipid()) {
            if gallery.len() >= MAX_GALLERY_ITEMS {
                break;
            }
            gallery.push(gallery_item(block));
            if gallery.len() % 16 == 0 {
                status.set(format!(
                    "Rendering structures… {}/{}",
                    gallery.len(),
                    lipid_count.min(MAX_GALLERY_ITEMS)
                ));
                TimeoutFuture::new(0).await;
            }
        }

        let summary = summarize(&blocks);
        let filtered_mgf = build_filtered_mgf(&blocks);

        analysis.set(Some(Analysis {
            summary,
            gallery,
            filtered_mgf,
            blocks,
        }));

        status.set(format!(
            "Selected {lipid_count} lipid spectra out of {total} (download below).",
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
    );
}

/// Trigger a browser download of `content` as `file_name`.
pub fn download_mgf(content: &str, file_name: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::console;

        let array = Array::new();
        array.push(&wasm_bindgen::JsValue::from(content));
        let blob = Blob::new_with_str_sequence(&array)
            .map_err(|_| "Failed to create a blob from the filtered MGF.".to_string())?;

        let url = Url::create_object_url_with_blob(&blob)
            .map_err(|_| "Failed to create a download URL.".to_string())?;
        let window = web_sys::window().ok_or_else(|| "No browser window.".to_string())?;
        let document = window
            .document()
            .ok_or_else(|| "No document object.".to_string())?;
        let anchor = document
            .create_element("a")
            .map_err(|_| "Failed to create the download anchor.".to_string())?
            .dyn_into::<HtmlAnchorElement>()
            .map_err(|_| "Failed to cast the anchor element.".to_string())?;
        anchor.set_href(&url);
        anchor.set_download(file_name);
        anchor.click();

        Url::revoke_object_url(&url)
            .map_err(|_| "Failed to revoke the download URL.".to_string())?;

        console::log_1(
            &format!(
                "lipid-selecto-rs: started download of {file_name} ({} bytes).",
                content.len()
            )
            .into(),
        );
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (content, file_name);
        Err("Download is only available in the browser (use `dx serve`).".to_string())
    }
}
