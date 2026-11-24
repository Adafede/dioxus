// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Unified error type for upload, file extraction, and browser download operations.

use stream_io::StreamError;
use thiserror::Error;

/// Error returned by `upload` streaming, extraction, and download operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum UploadError {
    /// Underlying byte or line streaming error.
    #[error(transparent)]
    Stream(#[from] StreamError),

    /// The operation is only available inside a browser environment.
    #[error("download is only available in the browser")]
    BrowserOnly,

    /// Triggering a browser download failed.
    #[error("download failed: {0}")]
    DownloadFailed(String),

    /// Extracting a `Blob` from a Dioxus `FileData` event failed.
    #[error("{0}")]
    FileExtraction(String),
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn format_js_value(value: &wasm_bindgen::JsValue) -> String {
    use wasm_bindgen::JsCast;
    if let Some(s) = value.as_string() {
        return s;
    }
    if let Some(err) = value.dyn_ref::<js_sys::Error>() {
        return String::from(err.message());
    }
    format!("{value:?}")
}

#[cfg(target_arch = "wasm32")]
impl From<wasm_bindgen::JsValue> for UploadError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        Self::Stream(StreamError::io(format_js_value(&value)))
    }
}

#[cfg(test)]
mod tests {
    use super::UploadError;
    use stream_io::StreamError;

    #[test]
    fn error_messages_are_clear() {
        assert_eq!(
            UploadError::BrowserOnly.to_string(),
            "download is only available in the browser"
        );
        assert_eq!(
            UploadError::DownloadFailed("no body".to_string()).to_string(),
            "download failed: no body"
        );
        assert_eq!(
            UploadError::FileExtraction("bad file".to_string()).to_string(),
            "bad file"
        );
        assert_eq!(
            UploadError::from(StreamError::LineTooLong { max_bytes: 128 }).to_string(),
            "line exceeds maximum length of 128 bytes"
        );
    }
}
