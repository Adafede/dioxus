// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Portable async chunked byte and line streaming primitives.
//!
//! Provides bounded-memory streaming readers over [`ChunkSource`] without
//! coupling to browser or UI frameworks:
//!
//! - [`BlobCursor`] — byte-level chunked lookahead and cursor advancement.
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

// `ChunkSource` implementations on wasm32 (such as browser `Blob` readers) are
// `!Send`, so generic streaming methods do not require `Send + Sync` bounds.
#![allow(clippy::future_not_send)]
#![warn(missing_docs)]

pub mod blob_cursor;
pub mod blob_lines;
mod buffer;
pub mod bytes;
mod error;
mod progress;

pub use blob_cursor::BlobCursor;
pub use blob_lines::{BlobLines, DEFAULT_MAX_LINE_BYTES};
pub use bytes::{ChunkSource, SliceSource};
pub use error::StreamError;
pub use progress::{PROGRESS_BYTE_INTERVAL, PROGRESS_TIME_INTERVAL_MS, ProgressThrottler};

/// Default chunk size per read (`16 MiB`).
pub const DEFAULT_CHUNK_SIZE: usize = 16 * 1024 * 1024;
