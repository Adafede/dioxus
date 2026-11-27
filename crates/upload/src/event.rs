// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Drag-and-drop and file-input event handling utilities.

use dioxus::html::FileData;
use dioxus::prelude::{ReadableExt, Signal, WritableExt, use_signal};

use crate::error::UploadError;

/// Monotonic generation token guarding async upload/analysis tasks against stale overwrites.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UploadGen(Signal<u64>);

impl UploadGen {
    /// Advances the generation counter and returns the new token.
    pub fn advance(&mut self) -> u64 {
        let next = self.0.peek().wrapping_add(1);
        self.0.set(next);
        next
    }

    /// Returns `true` while `expected` matches the latest generation token.
    #[must_use]
    pub fn is_current(&self, expected: u64) -> bool {
        *self.0.peek() == expected
    }
}

/// Hook creating an [`UploadGen`] counter initialized to `0`.
#[must_use]
pub fn use_upload_gen() -> UploadGen {
    UploadGen(use_signal(|| 0u64))
}

/// Records the selected file `name` and sets the browser-only notice in `status`.
pub fn report_browser_only_upload(
    mut file_name: Signal<String>,
    mut status: Signal<String>,
    name: String,
) {
    file_name.set(name);
    status.set("This app needs to run in a browser.".to_string());
}

/// Extracts the first uploaded file from `files`, writing selection/error status into `status`.
pub fn dispatch_file_upload(
    files: &[FileData],
    mut status: Signal<String>,
    on_file: impl FnOnce(ExtractedFile),
) {
    match extract_blob_from_file_data(files) {
        Ok(Some(file)) => on_file(file),
        Ok(None) => status.set("No file selected.".to_string()),
        Err(err) => status.set(err.to_string()),
    }
}

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
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(
        clippy::missing_const_for_fn,
        reason = "wasm32 branch uses non-const File/JsCast calls"
    )
)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::*;

    #[test]
    fn upload_gen_and_dispatch_report_empty_and_track_generations() {
        let mut dom = VirtualDom::new(|| {
            let mut upload_gen = use_upload_gen();
            let status = use_signal(String::new);
            let first = upload_gen.advance();
            assert_eq!(first, 1);
            assert!(upload_gen.is_current(first));
            let second = upload_gen.advance();
            assert_eq!(second, 2);
            assert!(!upload_gen.is_current(first));
            assert!(upload_gen.is_current(second));

            let mut invoked = false;
            dispatch_file_upload(&[], status, |_| invoked = true);
            assert!(!invoked);
            assert_eq!(&*status.read(), "No file selected.");
            let file_name = use_signal(String::new);
            report_browser_only_upload(file_name, status, "sample.json".to_string());
            assert_eq!(&*file_name.read(), "sample.json");
            assert_eq!(&*status.read(), "This app needs to run in a browser.");
            VNode::empty()
        });
        dom.rebuild_in_place();
    }
}
