// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Unified error type for all upload/read/download operations.
//!
//! Replaces the ad-hoc `ScanError(JsValue)` from the old `file-upload` crate
//! and the `String` error plumbing scattered across apps.  The WASM-specific
//! `JsValue` is converted to a human-readable string at the boundary so the
//! error type itself is platform-agnostic.

use thiserror::Error;

/// Error returned by `upload::BlobCursor` and `upload::BlobLines` operations.
#[derive(Debug, Error)]
pub enum UploadError {
    /// A browser/JS API call failed.  The inner string is the JS representation.
    #[error("blob read error: {0}")]
    Blob(String),

    /// The stream ended before the parser expected it to (truncated input).
    #[error("unexpected EOF while reading stream")]
    UnexpectedEof,

    /// A structural invariant of the input was violated.
    #[error("expected {expected}")]
    Expected {
        /// Human-readable description of what was expected.
        expected: &'static str,
    },

    /// The operation is only available inside the browser.
    #[error("download is only available in the browser")]
    BrowserOnly,

    /// A generic, formatted error for app-level validation.
    #[error("{0}")]
    Other(String),
}

impl UploadError {
    /// Convenience for "expected X but got end of stream".
    #[must_use]
    pub const fn expected(expected: &'static str) -> Self {
        Self::Expected { expected }
    }

    /// Convenience for wrapping a message.
    #[must_use]
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

#[cfg(target_arch = "wasm32")]
impl From<wasm_bindgen::JsValue> for UploadError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        use wasm_bindgen::JsCast;
        Self::Blob(value.dyn_ref::<js_sys::JsString>().map_or_else(
            || format!("{value:?}"),
            |s| s.as_string().unwrap_or_default(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages_are_clear() {
        assert_eq!(
            UploadError::UnexpectedEof.to_string(),
            "unexpected EOF while reading stream",
        );
        assert_eq!(
            UploadError::expected("opening quote").to_string(),
            "expected opening quote",
        );
        assert_eq!(UploadError::other("bad magic").to_string(), "bad magic",);
    }

    #[test]
    fn other_accepts_into_string() {
        let err = UploadError::other("oops");
        assert_eq!(err.to_string(), "oops");
    }
}
