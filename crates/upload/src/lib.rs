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
//! - **`extract_blob_from_form_data`** / **`extract_blob_from_web_file`** —
//!   unified file-input / drag-drop extraction, eliminating the identical inline
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

#[cfg(target_arch = "wasm32")]
mod blob_cursor;
#[cfg(target_arch = "wasm32")]
mod blob_lines;
mod download;
mod error;
mod event;
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
