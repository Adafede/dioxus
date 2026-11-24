// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! # upload — browser file I/O, extraction, and download adapters
//!
//! Browser-facing adapter over [`stream_io`] providing:
//!
//! - `BlobSource` (`wasm32`) — [`ChunkSource`] backed by a browser `Blob` with a
//!   monotonic `performance.now()` clock.
//! - [`extract_blob_from_file_data`] — unified file-input / drag-drop
//!   extraction over `&[FileData]`.
//! - [`download_text`] / [`download_svg`] / [`download_with_mime`] — browser
//!   download helpers returning [`UploadError`].
//! - Re-exports of [`BlobCursor`], [`BlobLines`], [`ChunkSource`],
//!   [`SliceSource`], and [`StreamError`] from [`stream_io`].

#![cfg_attr(target_arch = "wasm32", allow(clippy::future_not_send))]
#![warn(missing_docs)]

/// Browser `Blob` chunk source (`wasm32`) and re-exported `stream-io` source types.
pub mod bytes;
/// Download helpers (browser-triggered and native stubs).
mod download;
/// Unified error type for all upload, extraction, and download operations.
mod error;
/// Drag-and-drop / file-input event extraction.
mod event;

#[cfg(target_arch = "wasm32")]
pub use bytes::BlobSource;
pub use download::{download_svg, download_text, download_with_mime, sanitize_filename};
pub use error::UploadError;
pub use event::{Blob, ExtractedFile, extract_blob_from_file_data};
pub use stream_io::{BlobCursor, BlobLines, ChunkSource, SliceSource, StreamError};
