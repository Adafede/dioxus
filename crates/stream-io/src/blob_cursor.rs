// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Chunked, buffering reader with byte-level lookahead over a [`ChunkSource`].

use crate::DEFAULT_CHUNK_SIZE;
use crate::buffer::ChunkBuffer;
use crate::bytes::ChunkSource;
use crate::error::StreamError;
use crate::progress::{PROGRESS_BYTE_INTERVAL, PROGRESS_TIME_INTERVAL_MS, ProgressThrottler};

/// Buffered, chunked reader with byte-level access.
#[derive(Debug)]
pub struct BlobCursor<F, S: ChunkSource> {
    inner: ChunkBuffer<S>,
    total_bytes: u64,
    chunk_size: usize,
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
            inner: ChunkBuffer::new(source),
            total_bytes,
            chunk_size: DEFAULT_CHUNK_SIZE,
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

    /// Overrides the chunk size, clamped to at least 1 byte.
    #[must_use]
    pub fn with_chunk_size(mut self, bytes: usize) -> Self {
        self.chunk_size = bytes.max(1);
        self
    }

    /// Current position in the stream (including bytes in previous buffers).
    #[must_use]
    pub const fn processed(&self) -> u64 {
        self.processed_before_buf + self.inner.pos as u64
    }

    /// Total byte count passed to the constructor.
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Drops consumed bytes and pulls the next chunk from the source.
    ///
    /// Returns `Ok(true)` if buffered data remains, `Ok(false)` at end of stream.
    ///
    /// # Errors
    ///
    /// Returns [`StreamError`] if the underlying chunk read fails.
    pub async fn fill(&mut self) -> Result<bool, StreamError> {
        let consumed = self.inner.pos as u64;
        if consumed > 0 {
            self.inner.compact_all();
            self.processed_before_buf += consumed;
        }

        if self.eof {
            return Ok(!self.inner.buf.is_empty());
        }

        let Some((_, end)) = self
            .inner
            .pull_chunk(self.chunk_size, self.total_bytes)
            .await?
        else {
            self.eof = true;
            return Ok(!self.inner.buf.is_empty());
        };

        if end >= self.total_bytes {
            self.eof = true;
        }

        let processed = self.processed();
        self.inner
            .report_and_yield(&mut self.progress, processed, self.total_bytes)
            .await;

        Ok(true)
    }

    /// Ensures at least one byte is available in the buffer.
    ///
    /// # Errors
    ///
    /// Returns [`StreamError`] if reading the next chunk from the source fails.
    pub async fn ensure_any(&mut self) -> Result<bool, StreamError> {
        while self.inner.pos >= self.inner.buf.len() {
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
    /// Returns [`StreamError`] if reading the next chunk from the source fails.
    pub async fn peek(&mut self) -> Result<Option<u8>, StreamError> {
        if self.ensure_any().await? {
            Ok(self.inner.buf.get(self.inner.pos).copied())
        } else {
            Ok(None)
        }
    }

    /// Reads the next byte and advances the cursor.
    ///
    /// # Errors
    ///
    /// Returns [`StreamError`] if reading the next chunk from the source fails.
    pub async fn next_byte(&mut self) -> Result<Option<u8>, StreamError> {
        if self.ensure_any().await? {
            let byte = self.inner.buf.get(self.inner.pos).copied();
            if byte.is_some() {
                self.inner.pos += 1;
            }
            Ok(byte)
        } else {
            Ok(None)
        }
    }

    /// Skips ASCII JSON whitespace in the stream.
    ///
    /// # Errors
    ///
    /// Returns [`StreamError`] if reading the next chunk from the source fails.
    pub async fn skip_ws(&mut self) -> Result<(), StreamError> {
        loop {
            while self
                .inner
                .buf
                .get(self.inner.pos)
                .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
            {
                self.inner.pos += 1;
            }
            if self.inner.pos < self.inner.buf.len() {
                return Ok(());
            }
            if !self.fill().await? {
                return Ok(());
            }
        }
    }

    /// Returns the current position within the active buffer.
    #[must_use]
    pub const fn pos(&self) -> usize {
        self.inner.pos
    }

    /// Returns the byte at the current buffer position, if buffered.
    #[must_use]
    pub fn current_byte(&self) -> Option<u8> {
        self.inner.buf.get(self.inner.pos).copied()
    }

    /// Advances the cursor position by `n`, clamped to the active buffer length.
    pub fn advance(&mut self, n: usize) {
        self.inner.pos = self.inner.pos.saturating_add(n).min(self.inner.buf.len());
    }

    /// Returns a slice of the active buffer.
    #[must_use]
    pub fn buffer(&self) -> &[u8] {
        &self.inner.buf
    }
}

#[cfg(test)]
mod tests {
    use super::BlobCursor;
    use crate::bytes::SliceSource;
    use futures_executor::block_on;

    #[test]
    fn advance_saturates_at_buffer_length() {
        block_on(async {
            let mut cursor = BlobCursor::new(SliceSource::new(b"abc"), 3, |_, _| {});
            assert!(matches!(cursor.fill().await, Ok(true)));
            cursor.advance(1);
            assert_eq!(cursor.pos(), 1);
            cursor.advance(usize::MAX);
            assert_eq!(cursor.pos(), 3);
        });
    }

    #[test]
    fn peek_and_next_byte_cross_chunk_boundaries() -> Result<(), crate::StreamError> {
        block_on(async {
            let data = b"hello!";
            let mut cursor =
                BlobCursor::new(SliceSource::new(data), 6, |_, _| {}).with_chunk_size(0);
            let mut out = Vec::new();
            while let Some(peeked) = cursor.peek().await? {
                let next = cursor.next_byte().await?;
                assert_eq!(next, Some(peeked), "next_byte should match peeked byte");
                out.push(peeked);
            }
            assert_eq!(out, data, "all bytes should match input");
            assert_eq!(cursor.processed(), 6, "processed count should match input");
            assert_eq!(cursor.total_bytes(), 6, "total_bytes should match input");
            assert_eq!(cursor.next_byte().await?, None, "EOF should yield None");
            Ok(())
        })
    }

    #[test]
    fn skip_ws_crosses_chunks_and_stops_at_eof() -> Result<(), crate::StreamError> {
        block_on(async {
            let data = b"  \t\r\n  xyz  \n";
            let mut cursor = BlobCursor::new(SliceSource::new(data), data.len() as u64, |_, _| {})
                .with_chunk_size(2);
            cursor.skip_ws().await?;
            assert_eq!(cursor.peek().await?, Some(b'x'), "should stop at 'x'");
            for expected in *b"xyz" {
                assert_eq!(cursor.next_byte().await?, Some(expected), "consume xyz");
            }
            cursor.skip_ws().await?;
            assert_eq!(cursor.peek().await?, None, "trailing ws should reach EOF");
            Ok(())
        })
    }
}
