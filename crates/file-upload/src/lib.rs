// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Streaming file upload and blob handling primitives for WASM file upload apps.
//!
//! This crate provides reusable components for handling file uploads in Dioxus/WASM
//! applications, including:
//!
//! - Chunked streaming reads from browser Blobs
//! - Drag-and-drop event handling
//! - File input parsing utilities
//! - Progress reporting integration
//!
//! # Example
//!
//! ```ignore
//! use file_upload::{BlobCursor, extract_blob_from_form_data, download_text};
//! use shared::progress::ProgressThrottler;
//!
//! // Extract blob from file input event
//! if let Ok((Some(blob), name, Ok(()))) = extract_blob_from_form_data(evt.data())? {
//!     let thunker = ProgressThrottler::new(
//!         |processed, total| status.set(format!("{processed}/{total}")),
//!         js_sys::Date::now,
//!         4 * 1024 * 1024,
//!         120.0,
//!     );
//!     let mut cursor = BlobCursor::new(&blob, blob.size(), thunker);
//!     // ... streaming processing
//! }
//! ```

#[cfg(target_arch = "wasm32")]
mod blob_cursor;
#[cfg(target_arch = "wasm32")]
mod download;
#[cfg(target_arch = "wasm32")]
mod event;

#[cfg(target_arch = "wasm32")]
pub use blob_cursor::{BlobCursor, ScanError};
#[cfg(target_arch = "wasm32")]
pub use download::{SKIP_LINK_STYLE, download_text, download_text_as_blob};
#[cfg(target_arch = "wasm32")]
pub use event::{extract_blob_from_form_data, extract_blob_from_web_file};
