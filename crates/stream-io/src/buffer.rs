// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared buffer compaction and chunk refill helper for chunked readers.

use crate::bytes::ChunkSource;
use crate::error::StreamError;
use crate::progress::ProgressThrottler;

/// Shared buffered-chunk state used by both [`crate::BlobCursor`] and [`crate::BlobLines`].
#[derive(Debug)]
pub(crate) struct ChunkBuffer<S> {
    pub(crate) source: S,
    pub(crate) buf: Vec<u8>,
    pub(crate) pos: usize,
    pub(crate) source_read: u64,
}

impl<S: ChunkSource> ChunkBuffer<S> {
    #[must_use]
    pub(crate) const fn new(source: S) -> Self {
        Self {
            source,
            buf: Vec::new(),
            pos: 0,
            source_read: 0,
        }
    }

    /// Discards bytes before `pos` and resets `pos` to `0`.
    pub(crate) fn compact_all(&mut self) {
        if self.pos > 0 {
            if self.pos >= self.buf.len() {
                self.buf.clear();
            } else {
                self.buf.drain(..self.pos);
            }
            self.pos = 0;
        }
    }

    /// Compacts the buffer when more than half of it has been consumed.
    pub(crate) fn compact_half(&mut self) {
        if self.pos > self.buf.len() / 2 {
            self.buf.drain(..self.pos);
            self.pos = 0;
        }
    }

    /// Pulls the next `chunk_size` slice up to `total_limit` and updates `source_read`.
    ///
    /// Returns the `(start, end)` range read, or `None` when `source_read >= total_limit`.
    pub(crate) async fn pull_chunk(
        &mut self,
        chunk_size: usize,
        total_limit: u64,
    ) -> Result<Option<(u64, u64)>, StreamError> {
        let start = self.source_read;
        let end = start.saturating_add(chunk_size as u64).min(total_limit);
        if start >= end {
            return Ok(None);
        }

        let chunk = self.source.read_chunk(start, end).await?;
        let read_len = chunk.len() as u64;
        if read_len == 0 {
            return Err(StreamError::io(
                "unexpected EOF: source returned empty chunk",
            ));
        }
        if self.buf.is_empty() {
            self.buf = chunk;
        } else {
            self.buf.extend_from_slice(&chunk);
        }
        self.source_read = start.saturating_add(read_len);
        Ok(Some((start, self.source_read)))
    }

    /// Reports progress if due and yields to the scheduler when a report fired.
    pub(crate) async fn report_and_yield<F: FnMut(u64, u64)>(
        &self,
        progress: &mut ProgressThrottler<F, fn() -> f64>,
        reported_bytes: u64,
        total_bytes: u64,
    ) {
        if progress.maybe_report(reported_bytes, total_bytes) {
            self.source.yield_now().await;
        }
    }
}
