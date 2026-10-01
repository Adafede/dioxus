// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Line-oriented chunked reader for text-based file formats (MGF, SMILES/CSV).
//!
//! Like [`crate::BlobCursor`], `BlobLines` reads the blob in 16 MiB chunks and
//! only `.await`s when the buffer is exhausted.  The difference is that it
//! splits the stream into `\n`-delimited lines, which is the natural unit for
//! text-based formats such as MGF blocks, SMILES lists, and CSV records.

use crate::bytes::ChunkSource;
use crate::error::UploadError;
use crate::progress::ProgressThrottler;
use crate::progress::{PROGRESS_BYTE_INTERVAL, PROGRESS_TIME_INTERVAL_MS};

/// A line-oriented, chunked reader.
///
/// Yields `String` lines (without trailing `\n` or `\r`) one at a time via
/// [`next_line`](Self::next_line). Internally buffers one 16 MiB chunk.
///
/// Generic over its [`ChunkSource`], so the browser reads a `Blob` and a host
/// test reads a slice, through identical code.
#[derive(Debug)]
pub struct BlobLines<F, S: ChunkSource> {
    source: S,
    total_bytes: u64,
    offset: u64,
    buffer: Vec<u8>,
    buf_start: usize,
    chunk_size: usize,
    processed: u64,
    progress: ProgressThrottler<F, fn() -> f64>,
}

impl<F, S> BlobLines<F, S>
where
    F: FnMut(u64, u64),
    S: ChunkSource,
{
    /// Creates a new line reader over `source`.
    #[must_use]
    pub fn new(source: S, on_progress: F) -> Self {
        let total_bytes = source.total_bytes();
        Self {
            source,
            total_bytes,
            offset: 0,
            chunk_size: crate::blob_cursor::CHUNK_SIZE,
            buffer: Vec::with_capacity(crate::blob_cursor::CHUNK_SIZE),
            buf_start: 0,
            processed: 0,
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
    /// See [`BlobLines`]' reason: a line straddling a chunk boundary is the one
    /// thing a line reader can get wrong, and at 16 MiB no test can reach one.
    #[must_use]
    pub fn with_chunk_size(mut self, bytes: usize) -> Self {
        self.chunk_size = bytes.max(1);
        self
    }

    /// Total bytes in the source.
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Returns the next line from the blob, or `Ok(None)` at end-of-stream.
    ///
    /// # Errors
    ///
    /// Returns [`UploadError`] if reading the next chunk from the blob fails.
    pub async fn next_line(&mut self) -> Result<Option<String>, UploadError> {
        loop {
            // Try to extract a complete line from the current buffer.
            if let Some(line) = self.take_line_from_buffer() {
                return Ok(Some(line));
            }

            // No complete line yet — check for EOF. Whatever is left in the
            // buffer is the last line, unterminated; `take_line_from_buffer` has
            // already drained everything above `buf_start`.
            if self.offset >= self.total_bytes {
                if let Some(remaining) = self.buffer.get(self.buf_start..) {
                    let remaining = String::from_utf8_lossy(remaining).into_owned();
                    self.buf_start = self.buffer.len();
                    return Ok(Some(remaining));
                }
                return Ok(None);
            }

            self.load_next_chunk().await?;
        }
    }

    fn take_line_from_buffer(&mut self) -> Option<String> {
        let available = self.buffer.get(self.buf_start..)?;
        if let Some(pos) = available.iter().position(|b| *b == b'\n')
            && let Some(line_bytes) = available.get(..pos)
        {
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
        let end = (self.offset + self.chunk_size as u64).min(self.total_bytes);
        let chunk_bytes = self.source.read_chunk(start, end).await?;
        self.buffer.extend_from_slice(&chunk_bytes);
        self.offset = end;
        self.processed = self.processed.saturating_add((end - start).max(1));
        if self.progress.maybe_report(self.processed, self.total_bytes) {
            // Yield to the event loop so the UI stays responsive.
            self.source.yield_now().await;
        }
        Ok(())
    }
}
