// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Throttled progress reporting for long-running streaming work.

/// Throttles callbacks based on bytes processed and monotonic time elapsed.
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

    /// Invokes the callback if either byte or time threshold has elapsed.
    pub(crate) fn maybe_report(&mut self, processed: u64, total: u64) -> bool {
        let now = (self.time_fn)();
        if now < self.last_reported_time {
            self.last_reported_time = now;
        }
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

/// Default byte interval for progress reporting (`4 MiB`).
pub(crate) const PROGRESS_BYTE_INTERVAL: u64 = 4 * 1024 * 1024;

/// Default time interval for progress reporting (`120 ms`).
pub(crate) const PROGRESS_TIME_INTERVAL_MS: f64 = 120.0;

#[cfg(test)]
mod tests;
