// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Line-oriented chunked reader for text-based file formats.

use crate::DEFAULT_CHUNK_SIZE;
use crate::buffer::ChunkBuffer;
use crate::bytes::ChunkSource;
use crate::error::StreamError;
use crate::progress::{PROGRESS_BYTE_INTERVAL, PROGRESS_TIME_INTERVAL_MS, ProgressThrottler};

/// Default cap on a single buffered line (`64 MiB`).
pub(crate) const DEFAULT_MAX_LINE_BYTES: usize = 64 * 1024 * 1024;

/// A line-oriented, chunked reader yielding lines without trailing `\n` or `\r\n`.
#[derive(Debug)]
pub struct BlobLines<F, S: ChunkSource> {
    inner: ChunkBuffer<S>,
    /// Buffer index (`>= inner.pos`) up to which no `\n` has been seen yet.
    scan_offset: usize,
    total_bytes: u64,
    chunk_size: usize,
    max_line_bytes: usize,
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
            inner: ChunkBuffer::new(source),
            scan_offset: 0,
            total_bytes,
            chunk_size: DEFAULT_CHUNK_SIZE,
            max_line_bytes: DEFAULT_MAX_LINE_BYTES,
            progress: ProgressThrottler::new(
                on_progress,
                S::clock(),
                PROGRESS_BYTE_INTERVAL,
                PROGRESS_TIME_INTERVAL_MS,
            ),
        }
    }

    /// Overrides the chunk size, clamped to at least 1 byte.
    #[must_use]
    pub fn with_chunk_size(mut self, bytes: usize) -> Self {
        self.chunk_size = bytes.max(1);
        self
    }

    /// Overrides the maximum allowed line length in bytes, clamped to at least 1 byte.
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn with_max_line_bytes(mut self, bytes: usize) -> Self {
        self.max_line_bytes = if bytes == 0 { 1 } else { bytes };
        self
    }

    /// Total bytes in the source.
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Returns the next line from the stream, or `Ok(None)` at end-of-stream.
    ///
    /// # Errors
    ///
    /// Returns [`StreamError`] if reading the next chunk fails or a line
    /// exceeds the configured maximum byte length.
    pub async fn next_line(&mut self) -> Result<Option<String>, StreamError> {
        loop {
            if let Some(line) = self.take_line_from_buffer()? {
                return Ok(Some(line));
            }
            let buffered = self.inner.buf.get(self.inner.pos..).unwrap_or_default();
            // Exclude a trailing `\r` mid-stream because `\r\n` may straddle a chunk boundary.
            let pending_len =
                if self.inner.source_read < self.total_bytes && buffered.ends_with(b"\r") {
                    buffered.len().saturating_sub(1)
                } else {
                    buffered.len()
                };
            if pending_len > self.max_line_bytes {
                return Err(StreamError::LineTooLong {
                    max_bytes: self.max_line_bytes,
                });
            }

            if self.inner.source_read >= self.total_bytes {
                let remaining = self.inner.buf.get(self.inner.pos..).unwrap_or_default();
                if remaining.is_empty() {
                    return Ok(None);
                }
                let remaining = String::from_utf8_lossy(remaining).into_owned();
                self.inner.pos = self.inner.buf.len();
                self.scan_offset = self.inner.pos;
                return Ok(Some(remaining));
            }

            self.load_next_chunk().await?;
        }
    }

    #[inline]
    fn take_line_from_buffer(&mut self) -> Result<Option<String>, StreamError> {
        let search_start = self.scan_offset.max(self.inner.pos);
        let Some(search_slice) = self.inner.buf.get(search_start..) else {
            self.scan_offset = self.inner.buf.len();
            return Ok(None);
        };
        if let Some(rel_nl) = search_slice.iter().position(|b| *b == b'\n') {
            let nl_idx = search_start + rel_nl;
            let Some(line_bytes) = self.inner.buf.get(self.inner.pos..nl_idx) else {
                return Ok(None);
            };
            let content_bytes = line_bytes.strip_suffix(b"\r").unwrap_or(line_bytes);
            if content_bytes.len() > self.max_line_bytes {
                return Err(StreamError::LineTooLong {
                    max_bytes: self.max_line_bytes,
                });
            }
            let line = String::from_utf8_lossy(content_bytes).into_owned();
            self.inner.pos = nl_idx + 1;
            self.scan_offset = self.inner.pos;
            Ok(Some(line))
        } else {
            self.scan_offset = self.inner.buf.len();
            Ok(None)
        }
    }

    async fn load_next_chunk(&mut self) -> Result<(), StreamError> {
        self.inner.compact_all();
        self.scan_offset = self.inner.buf.len();
        if self
            .inner
            .pull_chunk(self.chunk_size, self.total_bytes)
            .await?
            .is_some()
        {
            self.inner
                .report_and_yield(&mut self.progress, self.inner.source_read, self.total_bytes)
                .await;
        }
        Ok(())
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests;
