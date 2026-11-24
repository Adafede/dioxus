// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Byte-source trait and host [`SliceSource`] implementation.

use std::fmt::Debug;
use std::time::Instant;

use crate::error::StreamError;

/// A source of bytes that chunked readers can pull ranges from.
#[expect(
    async_fn_in_trait,
    reason = "host and wasm32 targets share identical signatures without Send bounds"
)]
pub trait ChunkSource: Debug {
    /// Total bytes this source can supply.
    #[must_use]
    fn total_bytes(&self) -> u64;

    /// Reads the range `start..end`.
    ///
    /// Callers pass `end` clamped to [`total_bytes`](Self::total_bytes).
    ///
    /// # Errors
    ///
    /// Returns [`StreamError`] if the underlying read fails.
    async fn read_chunk(&self, start: u64, end: u64) -> Result<Vec<u8>, StreamError>;

    /// Lets the scheduler run other tasks between chunk reads.
    async fn yield_now(&self);

    /// Milliseconds on a monotonic clock, as a bare function pointer.
    #[must_use]
    fn clock() -> fn() -> f64;
}

/// A [`ChunkSource`] backed by an in-memory byte slice.
#[derive(Debug, Clone)]
pub struct SliceSource<'a> {
    bytes: &'a [u8],
}

impl<'a> SliceSource<'a> {
    /// Wraps a slice as a byte source.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}

impl ChunkSource for SliceSource<'_> {
    fn total_bytes(&self) -> u64 {
        self.bytes.len() as u64
    }

    #[expect(
        clippy::unused_async_trait_impl,
        reason = "an `async fn` in a trait impl must stay `async fn`; the trait's signature is the contract"
    )]
    async fn read_chunk(&self, start: u64, end: u64) -> Result<Vec<u8>, StreamError> {
        let start =
            usize::try_from(start).map_err(|_| StreamError::OffsetOutOfRange { offset: start })?;
        let end =
            usize::try_from(end).map_err(|_| StreamError::OffsetOutOfRange { offset: end })?;
        Ok(self.bytes.get(start..end).unwrap_or_default().to_vec())
    }

    async fn yield_now(&self) {}

    fn clock() -> fn() -> f64 {
        slice_clock
    }
}

fn slice_clock() -> f64 {
    thread_local! {
        static EPOCH: std::cell::Cell<Option<Instant>> = const { std::cell::Cell::new(None) };
    }
    EPOCH.with(|epoch| {
        let start = epoch.get().unwrap_or_else(|| {
            let now = Instant::now();
            epoch.set(Some(now));
            now
        });
        start.elapsed().as_secs_f64() * 1000.0
    })
}
