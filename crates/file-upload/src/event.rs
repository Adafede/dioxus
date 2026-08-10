// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Drag-and-drop and file input event handling utilities.
//!
//! Provides reusable handlers for common file upload UI patterns.

use dioxus::events::FormData;
use wasm_bindgen::JsCast;
use web_sys::{Blob, File as WebFile};

/// Extracts a `Blob` from a web_sys::File object.
///
/// Returns `Err` if the file cannot be converted to a Blob.
#[cfg(target_arch = "wasm32")]
pub fn extract_blob_from_web_file(file: &WebFile) -> Result<Blob, String> {
    let Ok(blob) = file.clone().dyn_into::<Blob>() else {
        return Err("Unable to read the selected file as a blob.".to_string());
    };
    Ok(blob)
}

/// Extracts file data from a FormData event (file input or drag/drop).
///
/// Returns a tuple with:
/// - Option<Blob>: The blob if extraction succeeded
/// - String: The filename
/// - Result<(), String>: Ok(()) on success, Err(message) on failure
///
/// # Example
///
/// ```ignore
/// let (blob, name, result) = extract_blob_from_form_data(evt.data())?;
/// match result {
///     Ok(()) => { /* process file */ },
///     Err(e) => { status.set(e); }
/// }
/// ```
#[cfg(target_arch = "wasm32")]
pub fn extract_blob_from_form_data(
    data: FormData,
) -> Result<(Option<Blob>, String, Result<(), String>), String> {
    let Some(file) = data.files().into_iter().next() else {
        return Ok((None, String::new(), Err("No file selected.".to_string())));
    };

    let file_name = file.name();

    // Get the inner value and try to downcast to web_sys::File
    let inner = file.inner();
    let Some(web_file) = inner.downcast_ref::<WebFile>() else {
        return Ok((
            None,
            file_name,
            Err("This file type is not supported in the browser.".to_string()),
        ));
    };

    match extract_blob_from_web_file(web_file) {
        Ok(blob) => Ok((Some(blob), file_name, Ok(()))),
        Err(e) => Ok((None, file_name, Err(e))),
    }
}
