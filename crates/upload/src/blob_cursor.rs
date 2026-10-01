// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Chunked, buffering readers over browser [`Blob`]s.
//!
//! Two complementary readers:
//!
//! - [`BlobCursor`] — byte-level access for binary or mixed-content formats
//!   where the parser needs random-ish byte lookahead.
//! - [`BlobLines`] — line-oriented access for text formats (MGF blocks,
//!   SMILES files, CSV) where the parser processes one `\n`-delimited line at
//!   a time.
//!
//! Both read the blob in 16 MiB chunks, keeping memory bounded regardless of
//! file size.  The only `.await` point is `fill()` / `next_line()`, called when
//! the current buffer is exhausted — never per-byte.

use crate::bytes::ChunkSource;
use crate::error::UploadError;
use crate::progress::{PROGRESS_BYTE_INTERVAL, PROGRESS_TIME_INTERVAL_MS, ProgressThrottler};

/// Default chunk size per read (16 MiB).
pub(crate) const CHUNK_SIZE: usize = 16 * 1024 * 1024;

/// Buffered, chunked reader with byte-level access.
///
/// Holds a single in-flight chunk (`buf[pos..]`) and only performs an async read
/// when that chunk is exhausted. All parsing happens synchronously on the buffer
/// content.
///
/// Generic over its [`ChunkSource`], so the browser reads a `Blob` and a host
/// test reads a slice, through identical code. `total_bytes` is a separate
/// constructor argument rather than the source's length because the browser
/// callers pass an upload total that is not the file's size — the denominator of
/// a progress bar is often a multi-file total.
#[derive(Debug)]
pub struct BlobCursor<F, S: ChunkSource> {
    source: S,
    total_bytes: u64,
    chunk_size: usize,
    source_read: u64,
    buf: Vec<u8>,
    pos: usize,
    processed_before_buf: u64,
    eof: bool,
    progress: ProgressThrottler<F, fn() -> f64>,
}

impl<F, S> BlobCursor<F, S>
where
    F: FnMut(u64, u64),
    S: ChunkSource,
{
    /// Creates a new cursor reading from `source` with progress reporting.
    #[must_use]
    pub fn new(source: S, total_bytes: u64, on_progress: F) -> Self {
        Self {
            source,
            total_bytes,
            chunk_size: CHUNK_SIZE,
            source_read: 0,
            buf: Vec::with_capacity(CHUNK_SIZE),
            pos: 0,
            processed_before_buf: 0,
            eof: false,
            progress: ProgressThrottler::new(
                on_progress,
                S::clock(),
                PROGRESS_BYTE_INTERVAL,
                PROGRESS_TIME_INTERVAL_MS,
            ),
        }
    }

    /// Overrides the chunk size, for testing.
    ///
    /// The reader's one real obligation is that it behaves identically whether a
    /// token falls inside one chunk or straddles two, and that obligation is
    /// untestable at 16 MiB — the only way to reach a boundary is to supply more
    /// data than any test should hold. A four-byte chunk puts boundaries
    /// everywhere a short input can reach, which is the only reason this exists.
    #[must_use]
    pub fn with_chunk_size(mut self, bytes: usize) -> Self {
        self.chunk_size = bytes.max(1);
        self
    }

    /// Current position in the stream (including bytes in previous buffers).
    #[must_use]
    pub const fn processed(&self) -> u64 {
        self.processed_before_buf + self.pos as u64
    }

    /// Total blob size in bytes.
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Drops consumed bytes and pulls the next chunk from the source.
    ///
    /// Returns `Ok(true)` if data is available to read, `Ok(false)` when the
    /// stream is fully exhausted and the buffer is empty.
    ///
    /// # Errors
    ///
    /// Returns [`UploadError`] if the underlying chunk read fails.
    pub async fn fill(&mut self) -> Result<bool, UploadError> {
        if self.pos > 0 {
            self.buf.drain(0..self.pos);
            self.processed_before_buf += self.pos as u64;
            self.pos = 0;
        }

        if self.eof {
            return Ok(!self.buf.is_empty());
        }

        let start = self.source_read;
        let end = (self.source_read + self.chunk_size as u64).min(self.total_bytes);
        if start >= end {
            self.eof = true;
            return Ok(!self.buf.is_empty());
        }

        let chunk = self.source.read_chunk(start, end).await?;
        self.buf.extend_from_slice(&chunk);

        self.source_read = end;
        if self.source_read >= self.total_bytes {
            self.eof = true;
        }

        if self
            .progress
            .maybe_report(self.processed(), self.total_bytes)
        {
            // Yield to the event loop only when progress is reported.
            self.source.yield_now().await;
        }

        Ok(true)
    }

    /// Ensures at least one byte is available in the buffer.
    ///
    /// # Errors
    ///
    /// Returns [`UploadError`] if reading the next chunk from the blob fails.
    pub async fn ensure_any(&mut self) -> Result<bool, UploadError> {
        while self.pos >= self.buf.len() {
            if !self.fill().await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Peeks at the next byte without consuming it.
    ///
    /// # Errors
    ///
    /// Returns [`UploadError`] if reading the next chunk from the blob fails.
    // `ensure_any` returning `true` *is* `self.pos < self.buf.len()`: it reads
    // until a byte is buffered and reports whether one is. Reading past that
    // would be a bug in this module rather than a runtime condition, and
    // returning `None` instead would report a truncated file as a complete one.
    #[allow(clippy::indexing_slicing)] // see above
    pub async fn peek(&mut self) -> Result<Option<u8>, UploadError> {
        if self.ensure_any().await? {
            Ok(Some(self.buf[self.pos]))
        } else {
            Ok(None)
        }
    }

    /// Reads the next byte and advances the cursor.
    ///
    /// # Errors
    ///
    /// Returns [`UploadError`] if reading the next chunk from the blob fails.
    // As in `peek`: `ensure_any` having returned `true` is the bound.
    #[allow(clippy::indexing_slicing)] // see `peek`
    pub async fn next_byte(&mut self) -> Result<Option<u8>, UploadError> {
        if self.ensure_any().await? {
            let b = self.buf[self.pos];
            self.pos += 1;
            Ok(Some(b))
        } else {
            Ok(None)
        }
    }

    /// Skips whitespace in the stream.
    ///
    /// # Errors
    ///
    /// Returns [`UploadError`] if reading the next chunk from the blob fails.
    pub async fn skip_ws(&mut self) -> Result<(), UploadError> {
        loop {
            while self
                .buf
                .get(self.pos)
                .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
            {
                self.pos += 1;
            }
            if self.pos < self.buf.len() {
                return Ok(());
            }
            if !self.fill().await? {
                return Ok(());
            }
        }
    }

    /// Returns the current position within the current buffer.
    #[must_use]
    pub const fn pos(&self) -> usize {
        self.pos
    }

    /// Returns the byte at the current position, if available.
    #[must_use]
    pub fn current_byte(&self) -> Option<u8> {
        self.buf.get(self.pos).copied()
    }

    /// Advances the cursor position by `n`, clamped to buffer length.
    pub fn advance(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.buf.len());
    }

    /// Returns a reference to the internal buffer.
    ///
    /// The buffer content is valid until the next call that modifies `pos`
    /// or calls `fill()`.
    #[must_use]
    pub fn buffer(&self) -> &[u8] {
        &self.buf
    }
}
