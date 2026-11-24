// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Drag-and-drop and file-input event handling utilities.

use dioxus::html::FileData;

use crate::error::UploadError;

/// Result of extracting a file from a form-data or drag-drop event.
#[derive(Debug)]
pub struct ExtractedFile {
    /// The file as a browser `Blob` ready for streaming reads.
    #[cfg(target_arch = "wasm32")]
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
/// # Errors
/// Returns [`UploadError::FileExtraction`] if the file cannot be downcast to a `Blob`.
#[allow(clippy::missing_const_for_fn)]
pub fn extract_blob_from_file_data(
    files: &[FileData],
) -> Result<Option<ExtractedFile>, UploadError> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;

        let Some(file) = files.first() else {
            return Ok(None);
        };

        let file_name = file.name();
        let Some(web_file) = file.inner().downcast_ref::<web_sys::File>() else {
            return Err(UploadError::FileExtraction(
                "This file type is not supported in the browser.".to_string(),
            ));
        };

        let blob = web_file.clone().dyn_into::<Blob>().map_err(|_| {
            UploadError::FileExtraction("Unable to read the selected file as a blob.".to_string())
        })?;
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
