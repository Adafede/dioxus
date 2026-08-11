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

use gloo_timers::future::sleep;
use js_sys::Uint8Array;
use wasm_bindgen_futures::JsFuture;
use web_sys::Blob;

use crate::error::UploadError;
use crate::progress::{PROGRESS_BYTE_INTERVAL, PROGRESS_TIME_INTERVAL_MS, ProgressThrottler};

/// Default chunk size for Blob reads (16 MiB).
pub const CHUNK_SIZE: usize = 16 * 1024 * 1024;

/// Buffered, chunked reader over a [`Blob`] with byte-level access.
///
/// Holds a single in-flight chunk (`buf[pos..]`) and only performs an async
/// Blob read when that chunk is exhausted.  All parsing happens synchronously
/// on the buffer content.
#[derive(Debug)]
pub struct BlobCursor<F> {
    blob: Blob,
    total_bytes: u64,
    blob_read: u64,
    buf: Vec<u8>,
    pos: usize,
    processed_before_buf: u64,
    eof: bool,
    progress: ProgressThrottler<F, fn() -> f64>,
}

impl<F> BlobCursor<F>
where
    F: FnMut(u64, u64),
{
    /// Creates a new cursor for reading from a `Blob` with progress reporting.
    #[must_use]
    pub fn new(blob: &Blob, total_bytes: u64, on_progress: F) -> Self {
        Self {
            blob: blob.clone(),
            total_bytes,
            blob_read: 0,
            buf: Vec::with_capacity(CHUNK_SIZE),
            pos: 0,
            processed_before_buf: 0,
            eof: false,
            progress: ProgressThrottler::new(
                on_progress,
                js_sys::Date::now,
                PROGRESS_BYTE_INTERVAL,
                PROGRESS_TIME_INTERVAL_MS,
            ),
        }
    }

    /// Current position in the stream (including bytes in previous buffers).
    #[must_use]
    pub fn processed(&self) -> u64 {
        self.processed_before_buf + self.pos as u64
    }

    /// Total blob size in bytes.
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Drops consumed bytes and pulls the next chunk from the blob.
    ///
    /// Returns `Ok(true)` if data is available to read, `Ok(false)` when the
    /// stream is fully exhausted and the buffer is empty.
    pub async fn fill(&mut self) -> Result<bool, UploadError> {
        if self.pos > 0 {
            self.buf.drain(0..self.pos);
            self.processed_before_buf += self.pos as u64;
            self.pos = 0;
        }

        if self.eof {
            return Ok(!self.buf.is_empty());
        }

        let start = self.blob_read;
        let end = (self.blob_read + CHUNK_SIZE as u64).min(self.total_bytes);
        if start >= end {
            self.eof = true;
            return Ok(!self.buf.is_empty());
        }

        let slice = self
            .blob
            .slice_with_f64_and_f64(start as f64, end as f64)
            .map_err(|e| UploadError::from(e))?;
        let array_buffer = JsFuture::from(slice.array_buffer()).await?;
        let array = Uint8Array::new(&array_buffer);

        let old_len = self.buf.len();
        let add_len = (end - start) as usize;
        self.buf.resize(old_len + add_len, 0);
        array.copy_to(&mut self.buf[old_len..old_len + add_len]);

        self.blob_read = end;
        if self.blob_read >= self.total_bytes {
            self.eof = true;
        }

        if self
            .progress
            .maybe_report(self.processed(), self.total_bytes)
        {
            // Yield to the event loop only when progress is reported.
            sleep(std::time::Duration::from_millis(0)).await;
        }

        Ok(true)
    }

    /// Ensures at least one byte is available in the buffer.
    pub async fn ensure_any(&mut self) -> Result<bool, UploadError> {
        while self.pos >= self.buf.len() {
            if !self.fill().await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Peeks at the next byte without consuming it.
    pub async fn peek(&mut self) -> Result<Option<u8>, UploadError> {
        if self.ensure_any().await? {
            Ok(Some(self.buf[self.pos]))
        } else {
            Ok(None)
        }
    }

    /// Reads the next byte and advances the cursor.
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
    pub async fn skip_ws(&mut self) -> Result<(), UploadError> {
        loop {
            while self.pos < self.buf.len()
                && matches!(self.buf[self.pos], b' ' | b'\t' | b'\n' | b'\r')
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

    /// Returns the total bytes read so far (same as [`processed`](Self::processed)).
    #[must_use]
    pub fn total_read(&self) -> u64 {
        self.processed()
    }

    /// Returns a reference to the internal buffer.
    ///
    /// The buffer content is valid until the next call that modifies `pos`
    /// or calls `fill()`.
    #[must_use]
    pub fn buffer(&self) -> &[u8] {
        &self.buf
    }

    /// Returns a mutable reference to the current position.
    pub fn pos_mut(&mut self) -> &mut usize {
        &mut self.pos
    }

    /// Forces a final progress report on the next [`fill`](Self::fill) call.
    pub fn force_progress_report(&mut self) {
        self.progress.force_next();
    }
}
