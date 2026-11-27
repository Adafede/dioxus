// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Portable async chunked byte and line streaming primitives.
//!
//! Provides bounded-memory streaming readers over [`ChunkSource`] without
//! coupling to browser or UI frameworks:
//!
//! - [`BlobCursor`] — byte-level chunked lookahead, slice access, and cursor advancement.
//! - [`BlobLines`] — line-oriented `\n`/`\r\n` streaming reader with line length caps.
//! - [`SliceSource`] — in-memory [`ChunkSource`] for host execution and tests.
//!
//! # Example
//!
//! ```rust
//! use futures_executor::block_on;
//! use stream_io::{BlobLines, SliceSource};
//!
//! let lines = block_on(async {
//!     let mut reader = BlobLines::new(SliceSource::new(b"alpha\nbeta\n"), |_, _| {});
//!     let mut out = Vec::new();
//!     while let Some(line) = reader.next_line().await? {
//!         out.push(line);
//!     }
//!     Ok::<_, stream_io::StreamError>(out)
//! });
//! assert_eq!(lines, Ok(vec!["alpha".to_string(), "beta".to_string()]));
//! ```

#![expect(
    clippy::future_not_send,
    reason = "ChunkSource implementations on wasm32 (such as browser Blob readers) are !Send"
)]
#![warn(missing_docs)]

mod blob_cursor;
mod blob_lines;
mod buffer;
mod bytes;
mod error;
mod progress;

pub use blob_cursor::BlobCursor;
pub use blob_lines::BlobLines;
pub use bytes::{ChunkSource, SliceSource};
pub use error::StreamError;

/// Default chunk size per read (`16 MiB`).
pub(crate) const DEFAULT_CHUNK_SIZE: usize = 16 * 1024 * 1024;
