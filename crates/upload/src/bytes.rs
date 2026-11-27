// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Browser `BlobSource` adapter implementing `stream_io::ChunkSource`.

use stream_io::{ChunkSource, StreamError};

/// A [`ChunkSource`] backed by a browser `Blob`.
#[derive(Debug, Clone)]
pub struct BlobSource {
    blob: web_sys::Blob,
}

impl BlobSource {
    /// Wraps a browser `Blob` as a byte source.
    #[must_use]
    pub fn new(blob: &web_sys::Blob) -> Self {
        Self { blob: blob.clone() }
    }

    /// Returns the byte length of the underlying browser `Blob`.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "DOM Blob::size is a non-negative integer stored as f64"
    )]
    #[must_use]
    pub fn total_bytes(&self) -> u64 {
        self.blob.size().max(0.0) as u64
    }
}

fn monotonic_clock_ms() -> f64 {
    web_sys::window()
        .and_then(|win| win.performance())
        .map_or_else(js_sys::Date::now, |perf| perf.now())
}

impl ChunkSource for BlobSource {
    fn total_bytes(&self) -> u64 {
        Self::total_bytes(self)
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "DOM Blob::slice takes f64 offsets (53-bit exact up to 8 PiB)"
    )]
    async fn read_chunk(&self, start: u64, end: u64) -> Result<Vec<u8>, StreamError> {
        let slice = self
            .blob
            .slice_with_f64_and_f64(start as f64, end as f64)
            .map_err(|err| StreamError::io(crate::error::format_js_value(&err)))?;
        let array_buffer = wasm_bindgen_futures::JsFuture::from(slice.array_buffer())
            .await
            .map_err(|err| StreamError::io(crate::error::format_js_value(&err)))?;
        let array = js_sys::Uint8Array::new(&array_buffer);
        Ok(array.to_vec())
    }

    async fn yield_now(&self) {
        gloo_timers::future::TimeoutFuture::new(0).await;
    }

    fn clock() -> fn() -> f64 {
        monotonic_clock_ms
    }
}
