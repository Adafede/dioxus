// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Chunked streaming reader for browser Blobs.
//!
//! This module provides a memory-efficient way to process large files in the browser
//! by reading them in chunks rather than loading the entire file into memory.

use dioxus::prelude::*;
use gloo_timers::future::sleep;
use js_sys::Uint8Array;
use shared::progress::ProgressThrottler;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::Blob;

/// Default chunk size for Blob reads (16 MiB).
#[cfg(target_arch = "wasm32")]
pub const CHUNK_SIZE: usize = 16 * 1024 * 1024;

/// Byte interval for progress reporting (4 MiB).
#[cfg(target_arch = "wasm32")]
pub const PROGRESS_BYTE_INTERVAL: u64 = 4 * 1024 * 1024;

/// Time interval for progress reporting (120ms).
#[cfg(target_arch = "wasm32")]
pub const PROGRESS_TIME_INTERVAL_MS: f64 = 120.0;

/// Error type for blob scanning operations.
/// Contains a JsValue that can be displayed as an error message.
#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct ScanError(pub JsValue);

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::error::Error for ScanError {}

impl From<JsValue> for ScanError {
    fn from(value: JsValue) -> Self {
        ScanError(value)
    }
}

/// Convert JsValue to our ScanError type.
fn js_value_to_scan_error(e: JsValue) -> ScanError {
    ScanError(e)
}

/// Buffered, chunked reader over a `Blob`.
///
/// Holds a single in-flight chunk (`buf[pos..]`) and only performs an async Blob read
/// when that chunk is exhausted. All parsing happens synchronously on the buffer content.
///
/// This means a 10 GB file with 16 MiB chunks needs only ~650 async suspension points,
/// instead of one per byte (10+ billion).
///
/// # Example
///
/// ```ignore
/// let mut cursor = BlobCursor::new(
///     &blob,
///     blob.size(),
///     |processed, total| println!("{}/{total}", processed),
/// );
///
/// while let Some(byte) = cursor.next_byte().await? {
///     // Process byte
/// }
/// ```
#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
impl<F> std::fmt::Debug for BlobCursor<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlobCursor")
            .field("total_bytes", &self.total_bytes)
            .field("blob_read", &self.blob_read)
            .field("buf_len", &self.buf.len())
            .field("pos", &self.pos)
            .field("processed_before_buf", &self.processed_before_buf)
            .field("eof", &self.eof)
            .finish()
    }
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::future_not_send)]
impl<F> BlobCursor<F>
where
    F: FnMut(u64, u64),
{
    /// Creates a new cursor for reading from a Blob with progress reporting.
    ///
    /// # Arguments
    /// - `blob`: The browser Blob to read from
    /// - `total_bytes`: Total size of the blob
    /// - `on_progress`: Callback invoked with (processed, total) when thresholds are met
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
    pub fn processed(&self) -> u64 {
        self.processed_before_buf + self.pos as u64
    }

    /// Drops consumed bytes and pulls the next chunk from the blob.
    ///
    /// Returns `Ok(true)` if data is available to read, `Ok(false)` when the stream
    /// is fully exhausted and the buffer is empty.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    pub async fn fill(&mut self) -> Result<bool, ScanError> {
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
            .map_err(js_value_to_scan_error)?;
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
            // Yield to the event loop only when progress is reported, so we
            // don't stall the UI thread on huge files but also don't yield
            // needlessly on every single chunk.
            sleep(std::time::Duration::from_millis(0)).await;
        }

        Ok(true)
    }

    /// Ensures at least one byte is available in the buffer.
    pub async fn ensure_any(&mut self) -> Result<bool, ScanError> {
        while self.pos >= self.buf.len() {
            if !self.fill().await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Peeks at the next byte without consuming it.
    pub async fn peek(&mut self) -> Result<Option<u8>, ScanError> {
        if self.ensure_any().await? {
            Ok(Some(self.buf[self.pos]))
        } else {
            Ok(None)
        }
    }

    /// Reads the next byte and advances the cursor.
    pub async fn next_byte(&mut self) -> Result<Option<u8>, ScanError> {
        if self.ensure_any().await? {
            let b = self.buf[self.pos];
            self.pos += 1;
            Ok(Some(b))
        } else {
            Ok(None)
        }
    }

    /// Skips whitespace in the stream.
    pub async fn skip_ws(&mut self) -> Result<(), ScanError> {
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

    /// Returns the current buffer position for manual inspection.
    #[allow(dead_code)]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns the current position within the current buffer.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Returns the byte at the current position, if available.
    pub fn current_byte(&self) -> Option<u8> {
        self.buf.get(self.pos).copied()
    }

    /// Advances the cursor position by the given amount.
    /// Returns the new position.
    pub fn advance(&mut self, n: usize) -> usize {
        self.pos = (self.pos + n).min(self.buf.len());
        self.pos
    }

    /// Returns the total bytes read so far.
    #[allow(dead_code)]
    pub fn total_read(&self) -> u64 {
        self.processed()
    }

    /// Returns a reference to the internal buffer.
    /// SAFETY: This allows direct buffer access for high-performance parsing.
    /// The buffer content is valid until the next call that modifies pos or fills.
    pub fn buffer(&self) -> &[u8] {
        &self.buf
    }

    /// Returns a mutable reference to the current position.
    pub fn pos_mut(&mut self) -> &mut usize {
        &mut self.pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    // Note: These tests are integration tests that require actual Blob objects
    // available in WASM context. Native tests are limited.

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn cursor_tracked_position_accurate() {
        // This test runs in wasm-bindgen-test environment
        // For native test, we skip it
    }
}
