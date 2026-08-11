// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Line-oriented chunked reader for text-based file formats (MGF, SMILES/CSV).
//!
//! Like [`crate::BlobCursor`], `BlobLines` reads the blob in 16 MiB chunks and
//! only `.await`s when the buffer is exhausted.  The difference is that it
//! splits the stream into `\n`-delimited lines, which is the natural unit for
//! text-based formats such as MGF blocks, SMILES lists, and CSV records.

use gloo_timers::future::TimeoutFuture;
use js_sys::Uint8Array;
use wasm_bindgen_futures::JsFuture;
use web_sys::Blob;

use crate::error::UploadError;
use crate::progress::ProgressThrottler;
use crate::progress::{PROGRESS_BYTE_INTERVAL, PROGRESS_TIME_INTERVAL_MS};

/// A line-oriented, chunked reader over a browser [`Blob`].
///
/// Yields `String` lines (without trailing `\n` or `\r`) one at a time via
/// [`next_line`](Self::next_line).  Internally buffers one 16 MiB chunk.
#[derive(Debug)]
pub struct BlobLines<F> {
    blob: Blob,
    total_bytes: u64,
    offset: u64,
    buffer: Vec<u8>,
    buf_start: usize,
    processed: u64,
    progress: ProgressThrottler<F, fn() -> f64>,
}

impl<F> BlobLines<F>
where
    F: FnMut(u64, u64),
{
    /// Creates a new line reader for the given blob.
    #[must_use]
    pub fn new(blob: &Blob, on_progress: F) -> Self {
        Self {
            blob: blob.clone(),
            total_bytes: blob.size() as u64,
            offset: 0,
            buffer: Vec::with_capacity(crate::blob_cursor::CHUNK_SIZE),
            buf_start: 0,
            processed: 0,
            progress: ProgressThrottler::new(
                on_progress,
                js_sys::Date::now,
                PROGRESS_BYTE_INTERVAL,
                PROGRESS_TIME_INTERVAL_MS,
            ),
        }
    }

    /// Total blob size in bytes.
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Returns the next line from the blob, or `Ok(None)` at end-of-stream.
    pub async fn next_line(&mut self) -> Result<Option<String>, UploadError> {
        loop {
            // Try to extract a complete line from the current buffer.
            if let Some(line) = self.take_line_from_buffer() {
                return Ok(Some(line));
            }

            // No complete line yet — check for EOF.
            if self.offset >= self.total_bytes {
                if self.buf_start < self.buffer.len() {
                    let remaining =
                        String::from_utf8_lossy(&self.buffer[self.buf_start..]).into_owned();
                    self.buf_start = self.buffer.len();
                    return Ok(Some(remaining));
                }
                return Ok(None);
            }

            self.load_next_chunk().await?;
        }
    }

    fn take_line_from_buffer(&mut self) -> Option<String> {
        let available = &self.buffer[self.buf_start..];
        if let Some(pos) = available.iter().position(|b| *b == b'\n') {
            let line_bytes = &available[..pos];
            let mut line = String::from_utf8_lossy(line_bytes).into_owned();
            self.buf_start += pos + 1;
            if line.ends_with('\r') {
                line.pop();
            }
            // Compact buffer when it's more than half consumed.
            if self.buf_start > self.buffer.len() / 2 {
                self.buffer.drain(..self.buf_start);
                self.buf_start = 0;
            }
            Some(line)
        } else {
            None
        }
    }

    async fn load_next_chunk(&mut self) -> Result<(), UploadError> {
        let start = self.offset;
        let end = (self.offset + crate::blob_cursor::CHUNK_SIZE as u64).min(self.total_bytes);
        let slice = self
            .blob
            .slice_with_f64_and_f64(start as f64, end as f64)
            .map_err(UploadError::from)?;
        let bytes = JsFuture::from(slice.array_buffer()).await?;
        let array = Uint8Array::new(&bytes);
        let chunk_len = array.byte_length() as usize;
        let mut chunk_bytes = vec![0u8; chunk_len];
        array.copy_to(&mut chunk_bytes);
        self.buffer.extend_from_slice(&chunk_bytes);
        self.offset = end;
        self.processed = self.processed.saturating_add((end - start).max(1));
        if self.progress.maybe_report(self.processed, self.total_bytes) {
            // Yield to the event loop so the UI stays responsive.
            TimeoutFuture::new(0).await;
        }
        Ok(())
    }
}
