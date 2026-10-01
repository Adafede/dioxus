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
//!   extraction over `&[FileData]`.
//! - **`download_text`** — browser download of text content.
//!
//! ## Design non-goals
//!
//! - Native file I/O (WASM-only by design)
//! - HTTP upload to servers
//! - SPARQL querying or LOTUS domain modeling → see the `lotus-search` crate

#![cfg_attr(target_arch = "wasm32", allow(clippy::future_not_send))]
#![warn(missing_docs)]

/// Byte-level chunked reader.
///
/// Not wasm-gated, and that is the point: it reads from a
/// [`bytes::ChunkSource`], so it compiles — and can be tested —
/// anywhere. Gating it on `cfg(test)` would not have worked, because `cfg(test)`
/// applies to this crate's own unit tests and not to a downstream crate's, which
/// is where the parser that needed testing lives.
pub mod blob_cursor;
/// Line-oriented chunked reader. See [`blob_cursor`].
pub mod blob_lines;
/// Where the chunked readers get their bytes.
pub mod bytes;
/// Download helpers (browser-triggered and native stubs).
mod download;
/// Unified error type for all upload operations.
mod error;
/// Drag-and-drop / file-input event extraction.
mod event;
/// Throttled progress reporting.
///
/// Portable: the clock is injected as a `fn() -> f64` rather than being
/// `js_sys::Date::now`, which is what let this move off wasm.
pub mod progress;

pub use blob_cursor::BlobCursor;
pub use blob_lines::BlobLines;
#[cfg(target_arch = "wasm32")]
pub use bytes::BlobSource;
pub use bytes::{ChunkSource, SliceSource};
pub use download::download_text;
pub use error::UploadError;
pub use event::{Blob, ExtractedFile, extract_blob_from_file_data};
