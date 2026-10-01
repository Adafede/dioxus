// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Where the chunked readers get their bytes.
//!
//! Named `bytes` rather than `source` or `chunk` because `ChunkSource` ends in
//! `Source` and starts with `Chunk`, and a module called either of those trips
//! `clippy::module_inception` from one side or the other.
//!
//! [`BlobCursor`](crate::BlobCursor) and [`BlobLines`](crate::BlobLines) hold one
//! chunk at a time and pull the next when the current one runs out, which is the
//! whole reason a multi-gigabyte upload costs 16 MiB of memory rather than the
//! size of the file.
//!
//! Where those chunks come from was a browser [`Blob`] and nothing else. That made
//! both readers impossible to compile outside wasm, and so left the parsing built
//! on top of them untested: `json-count-rs`'s streaming JSON scanner — the one
//! that runs in the browser, over multi-gigabyte uploads — had no host test
//! reachable at all, while a separate in-memory implementation of the same
//! grammar sat next to it under full test. Two implementations, one covered, and
//! no compiler error to say so.
//!
//! [`ChunkSource`] is that seam. It is four methods, and only
//! [`read_chunk`](ChunkSource::read_chunk) does any real work. The clock and the
//! yield are here because keeping the event loop responsive and throttling
//! progress reports are browser concerns, and a host test should not have to
//! reimplement either in order to compile the parsing it is there to test.
//!
//! Two implementations: `BlobSource` over a browser `Blob`, and [`SliceSource`]
//! over bytes that are already in memory. `BlobSource` is compiled for wasm only,
//! so it cannot be linked from a doc comment built on the host — which is the
//! clearest statement of what this seam is for.

use std::fmt::Debug;
use std::time::Instant;

use crate::error::UploadError;

/// A source of bytes the chunked readers can pull ranges from.
///
/// Implemented by `BlobSource` for a browser `Blob` and by [`SliceSource`] for
/// a slice already in memory. The trait exists so that the readers — and every
/// parser written against them — can be compiled and tested on the host.
// `async fn` in a trait is here on purpose. The lint's objection is that a
// caller cannot name the future type or ask for it to be `Send`, and the honest
// answer is that neither is wanted: every implementation's future is `!Send`
// already on wasm (the crate allows `future_not_send` for exactly that reason),
// and adding a `Send` bound would be false there while omitting it is merely
// permissive on the host. Every caller in this workspace immediately `.await`s.
#[allow(async_fn_in_trait)] // see above
pub trait ChunkSource: Debug {
    /// Total bytes this source can supply.
    fn total_bytes(&self) -> u64;

    /// Reads the range `start..end`.
    ///
    /// Callers pass `end` already clamped to [`total_bytes`](Self::total_bytes),
    /// and expect exactly `end - start` bytes back. Returning fewer would be read
    /// as end-of-stream by both readers, so a source with a short read must pad
    /// rather than truncate.
    ///
    /// # Errors
    ///
    /// Returns [`UploadError`] if the underlying read fails.
    async fn read_chunk(&self, start: u64, end: u64) -> Result<Vec<u8>, UploadError>;

    /// Lets the scheduler run other tasks.
    ///
    /// The browser implementations yield so that reading a large file does not
    /// freeze the page. [`SliceSource`] completes inline: there is no event loop
    /// on a host test to starve, and yielding would only make tests slow.
    async fn yield_now(&self);

    /// Milliseconds on a monotonic clock, as a bare function pointer.
    ///
    /// A pointer rather than a closure so the progress throttler can hold it
    /// without borrowing the chunk source, which it would otherwise have to do while
    /// living inside the same struct as that source.
    fn clock() -> fn() -> f64;
}

/// A [`ChunkSource`] over bytes that are already in memory.
///
/// The host-side implementation: same chunking, same buffering, same progress
/// reporting as the browser reader, but no `Blob`. Every parser built on
/// [`BlobCursor`](crate::BlobCursor) can be tested through this, which is what
/// finally reaches the code that only ever compiled to wasm.
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
        // A `usize` cannot exceed `u64`, so this is a widening cast, not a
        // truncation. The alternative — `try_into().unwrap_or(u64::MAX)` — would
        // claim a length that cannot exist on this target.
        self.bytes.len() as u64
    }

    // The trait method is `async fn` in a trait, so the impl is `async fn` too.
    // Clippy is right that this one never suspends, and wrong about the fix:
    // returning `impl Future` here to satisfy it would make the one host
    // implementation of the seam the only one whose signature differs from the
    // trait's, which is the property the seam exists to guarantee.
    #[expect(
        clippy::unused_async_trait_impl,
        reason = "an `async fn` in a trait impl must stay `async fn`; the trait's signature is the contract"
    )]
    async fn read_chunk(&self, start: u64, end: u64) -> Result<Vec<u8>, UploadError> {
        // `u64 -> usize` is fallible on wasm32 and a silent truncation there would
        // slice at the wrong offset, so it is checked rather than cast. The
        // reader only ever passes offsets it computed from `total_bytes`, which
        // came from this slice's own length, so this cannot actually fail.
        let start = usize::try_from(start).map_err(|_| {
            UploadError::other(format!("chunk offset {start} does not fit a usize"))
        })?;
        let end = usize::try_from(end)
            .map_err(|_| UploadError::other(format!("chunk end {end} does not fit a usize")))?;
        Ok(self.bytes.get(start..end).unwrap_or_default().to_vec())
    }

    // No `#[expect]` here, unlike `read_chunk` above: an empty `async` body is
    // not reported by `unused_async_trait_impl`, so an expectation would be
    // unfulfilled and would itself be a warning.
    async fn yield_now(&self) {
        // Nothing to yield to. See the trait method.
    }

    fn clock() -> fn() -> f64 {
        slice_clock
    }
}

/// The monotonic clock [`SliceSource`] hands to the progress throttler.
///
/// `thread_local` rather than a `static`: a `static` would need `Sync`, and an
/// `Instant` is not. It is read at most a few times per chunk, so the borrow
/// check on each access costs nothing measurable.
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

/// A [`ChunkSource`] over a browser `Blob`.
///
/// The only part of the chunked readers that needs a browser, which is what
/// makes it the only part that is compiled for wasm alone.
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct BlobSource {
    blob: web_sys::Blob,
}

#[cfg(target_arch = "wasm32")]
impl BlobSource {
    /// Wraps a browser `Blob` as a byte source.
    #[must_use]
    pub fn new(blob: &web_sys::Blob) -> Self {
        Self { blob: blob.clone() }
    }
}

#[cfg(target_arch = "wasm32")]
impl ChunkSource for BlobSource {
    // `Blob::size` is an `f64` because that is what the DOM says, and it is an
    // integer count in practice. Both casts below are the DOM's shape and cannot
    // be expressed any other way; the `.max(0.0)` is what makes the sign-loss
    // allow honest, since the negative case is excluded before the cast rather
    // than clamped afterwards.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // see above
    fn total_bytes(&self) -> u64 {
        self.blob.size().max(0.0) as u64
    }

    // `Blob::slice` takes `f64` bounds because that is what the DOM's `Blob`
    // takes, so the `u64` offsets are widened to it. Widening is exact; it is the
    // narrowing back inside the browser that the DOM does, not us.
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)] // see above
    async fn read_chunk(&self, start: u64, end: u64) -> Result<Vec<u8>, UploadError> {
        let slice = self
            .blob
            .slice_with_f64_and_f64(start as f64, end as f64)
            .map_err(UploadError::from)?;
        let array_buffer = wasm_bindgen_futures::JsFuture::from(slice.array_buffer()).await?;
        let array = js_sys::Uint8Array::new(&array_buffer);
        // `byte_length` rather than `end - start`: the Blob is the authority on
        // how much it gave back, and trusting the arithmetic instead would hide
        // a short read behind a zero-filled tail.
        let mut bytes = vec![0u8; array.byte_length() as usize];
        array.copy_to(&mut bytes);
        Ok(bytes)
    }

    async fn yield_now(&self) {
        // Zero milliseconds, which is the browser's way of saying "let the event
        // loop run whatever is queued and then carry on". It resolves to `()`.
        gloo_timers::future::TimeoutFuture::new(0).await;
    }

    fn clock() -> fn() -> f64 {
        js_sys::Date::now
    }
}
