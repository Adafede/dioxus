// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Throttled progress reporting for long-running browser-side work.
//!
//! One implementation, used by both chunked readers, so no app has to maintain
//! its own byte-count / time-throttle logic. Apps never name this type; they
//! pass an `on_progress` closure to `BlobCursor::new` or `BlobLines::new`.
//!
//! # Throttling strategy
//!
//! The callback fires when **either** threshold is exceeded since the last
//! report:
//! - `byte_threshold` bytes have been processed, **or**
//! - `time_threshold_ms` milliseconds have elapsed.
//!
//! This prevents UI thread flooding while still giving the user timely
//! feedback on large files.

/// Throttles callbacks based on bytes processed and wall-clock time elapsed.
///
/// Useful for streaming operations (file reading, parsing) where you want
/// progress updates without flooding the callback.
///
/// # Example (WASM)
/// ```ignore
/// let mut reporter = ProgressThrottler::new(
///     |processed, total| status.set(format!("{processed}/{total}")),
///     js_sys::Date::now,
///     4 * 1024 * 1024, // report every 4 MiB
///     120.0,            // or every 120 ms
/// );
/// reporter.maybe_report(bytes_read, total_size);
/// ```
#[derive(Debug)]
pub(crate) struct ProgressThrottler<F, T> {
    last_reported_bytes: u64,
    last_reported_time: f64,
    callback: F,
    time_fn: T,
    byte_threshold: u64,
    time_threshold_ms: f64,
}

impl<F, T> ProgressThrottler<F, T>
where
    F: FnMut(u64, u64),
    T: Fn() -> f64,
{
    /// Creates a new throttler with the given callback and time source.
    ///
    /// # Arguments
    /// - `callback`: Called with `(bytes_processed, total_bytes)` when thresholds are met
    /// - `time_fn`: Returns current time in milliseconds (e.g. `js_sys::Date::now`)
    /// - `byte_threshold`: Report after processing at least this many bytes
    /// - `time_threshold_ms`: Report after at least this many milliseconds
    #[must_use]
    pub(crate) fn new(
        callback: F,
        time_fn: T,
        byte_threshold: u64,
        time_threshold_ms: f64,
    ) -> Self {
        let now = time_fn();
        Self {
            last_reported_bytes: 0,
            last_reported_time: now,
            callback,
            time_fn,
            byte_threshold,
            time_threshold_ms,
        }
    }

    /// Invokes the callback with `(processed, total)` and returns `true` if
    /// enough bytes or time has elapsed since the last report, otherwise
    /// returns `false` without calling the callback.
    ///
    /// # Complexity
    /// O(1): single comparison and possible callback invocation.
    pub(crate) fn maybe_report(&mut self, processed: u64, total: u64) -> bool {
        let now = (self.time_fn)();
        let bytes_delta = processed.saturating_sub(self.last_reported_bytes);

        if bytes_delta >= self.byte_threshold
            || now - self.last_reported_time >= self.time_threshold_ms
        {
            (self.callback)(processed, total);
            self.last_reported_bytes = processed;
            self.last_reported_time = now;
            true
        } else {
            false
        }
    }
}

/// Default byte interval for progress reporting (4 MiB).
pub(crate) const PROGRESS_BYTE_INTERVAL: u64 = 4 * 1024 * 1024;

/// Default time interval for progress reporting (120 ms).
pub(crate) const PROGRESS_TIME_INTERVAL_MS: f64 = 120.0;

#[cfg(test)]
mod tests;
