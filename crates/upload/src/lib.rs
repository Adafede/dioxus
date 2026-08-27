// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! # upload — streaming file I/O, progress, and download for WASM apps
//!
//! The single crate that every upload-based WASM app in this workspace
//! consumes.  It provides:
//!
//! - `BlobCursor` — byte-level chunked streaming over a browser `Blob`,
//!   keeping exactly one 16 MiB chunk in memory regardless of file size.
//! - `BlobLines` — line-oriented chunked streaming for text formats (MGF,
//!   SMILES, CSV).
//! - **`ProgressThrottler`** — byte+time throttled progress callbacks, shared by
//!   all upload apps.
//! - **`extract_blob_from_file_data`** — unified file-input / drag-drop
//!   extraction over `&[FileData]`, eliminating the identical inline
//!   boilerplate every app previously copied.
//!
//! - **Download helpers** — `download_text`, `download_text_as_blob`,
//!   `download_url`, and `submit_download_form`, consolidating the per-app
//!   download code from `json-count-rs`, `mgf-precursor-erro-rs`,
//!   `lipid-selecto-rs`, and `lotus-explore-rs`.
//!
//! ## Design non-goals
//!
//! - Native file I/O (WASM-only by design)
//! - HTTP upload to servers
//! - SPARQL querying or LOTUS domain modeling → see the `lotus` crate

#![cfg_attr(target_arch = "wasm32", allow(clippy::future_not_send))]
#![warn(missing_docs)]

/// WASM-only: byte-level chunked reader over a browser `Blob`.
#[cfg(target_arch = "wasm32")]
mod blob_cursor;
/// WASM-only: line-oriented chunked reader over a browser `Blob`.
#[cfg(target_arch = "wasm32")]
mod blob_lines;
/// Download helpers (browser-triggered and native stubs).
mod download;
/// Unified error type for all upload operations.
mod error;
/// Drag-and-drop / file-input event extraction.
mod event;
/// Throttled progress reporting.
mod progress;

#[cfg(target_arch = "wasm32")]
pub use blob_cursor::{BlobCursor, CHUNK_SIZE};
#[cfg(target_arch = "wasm32")]
pub use blob_lines::BlobLines;
pub use download::{
    SKIP_LINK_STYLE, download_text, download_text_as_blob, download_url, sanitize_filename,
    submit_download_form,
};
pub use error::UploadError;
pub use event::{Blob, ExtractedFile, extract_blob_from_file_data};
pub use progress::{PROGRESS_BYTE_INTERVAL, PROGRESS_TIME_INTERVAL_MS, ProgressThrottler};

/// Read a blob as a string using streaming (handles large files without OOM).
///
/// This uses [`BlobLines`] internally to stream the file in 16 MiB chunks,
/// avoiding the memory issues of [`FileReader::read_as_text()`] which loads
/// the entire file into memory at once.
///
/// # Errors
/// Returns an error if the blob cannot be read or contains invalid UTF-8.
#[cfg(target_arch = "wasm32")]
pub async fn read_blob_string(blob: &Blob) -> Result<String, UploadError> {
    let mut reader = BlobLines::new(blob, |_, _| {});
    let mut out = String::new();
    while let Some(line) = reader.next_line().await? {
        out.push_str(&line);
        out.push('\n');
    }
    Ok(out)
}

/// Non-WASM stub for `read_blob_string`.
#[cfg(not(target_arch = "wasm32"))]
pub async fn read_blob_string(_blob: &Blob) -> Result<String, UploadError> {
    Err(UploadError::other("read_blob_string only available on WASM targets"))
}
