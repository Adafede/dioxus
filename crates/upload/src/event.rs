// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Drag-and-drop and file-input event handling utilities.
//!
//! Every upload-based app needs the same three lines:
//! 1. Extract the first file from the event's file list.
//! 2. Downcast to `web_sys::File`.
//! 3. Convert to a `Blob`.
//!
//! This module is the *single* implementation.  Apps call
//! [`extract_blob_from_file_data`] and pattern match on the result.

use dioxus::html::FileData;

/// Result of extracting a file from a form-data or drag-drop event.
#[derive(Debug)]
pub struct ExtractedFile {
    /// The file as a browser `Blob` ready for streaming reads.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub blob: Blob,
    /// The original filename from the `<input>` or drag source.
    pub name: String,
}

/// The browser `Blob` type (alias to `web_sys::Blob` on WASM; `()` on native).
#[cfg(target_arch = "wasm32")]
pub type Blob = web_sys::Blob;

/// Placeholder type on non-WASM targets where browser APIs are unavailable.
#[cfg(not(target_arch = "wasm32"))]
pub type Blob = ();

/// Extracts a [`Blob`] from the first `FileData` in a list.
///
/// Callers pass the result of `evt.data().files()` (which works for both
/// file-input `FormData` events and drag-drop `DragData` events).
///
/// # Errors
/// Returns a string message if the file cannot be downcast to a `Blob`.
#[allow(clippy::missing_const_for_fn)] // WASM path uses non-const JsCast trait methods
pub fn extract_blob_from_file_data(files: &[FileData]) -> Result<Option<ExtractedFile>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        type WebFile = web_sys::File;

        let Some(file) = files.iter().next() else {
            return Ok(None);
        };

        let file_name = file.name();
        let Some(web_file) = file.inner().downcast_ref::<WebFile>() else {
            return Err("This file type is not supported in the browser.".to_string());
        };

        let blob = web_file
            .clone()
            .dyn_into::<Blob>()
            .map_err(|_| "Unable to read the selected file as a blob.".to_string())?;
        Ok(Some(ExtractedFile {
            blob,
            name: file_name,
        }))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = files;
        Ok(None)
    }
}
