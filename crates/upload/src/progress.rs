// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Throttled progress reporting for long-running browser-side work.
//!
//! A single [`ProgressThrottler`] instance is shared by all upload apps to
//! avoid each app maintaining its own byte-count / time-throttle logic.
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
pub struct ProgressThrottler<F, T> {
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
    pub fn new(callback: F, time_fn: T, byte_threshold: u64, time_threshold_ms: f64) -> Self {
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
    pub fn maybe_report(&mut self, processed: u64, total: u64) -> bool {
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

    /// Forces the next call to [`maybe_report`](Self::maybe_report) to report
    /// immediately, regardless of thresholds.  Useful for flushing at end of
    /// stream.
    pub const fn force_next(&mut self) {
        self.last_reported_bytes = u64::MAX;
        self.last_reported_time = f64::NEG_INFINITY;
    }
}

/// Default byte interval for progress reporting (4 MiB).
pub const PROGRESS_BYTE_INTERVAL: u64 = 4 * 1024 * 1024;

/// Default time interval for progress reporting (120 ms).
pub const PROGRESS_TIME_INTERVAL_MS: f64 = 120.0;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn throttler_reports_when_byte_threshold_exceeded() {
        let reports = Rc::new(RefCell::new(Vec::new()));
        let reports_clone = reports.clone();
        let time = Rc::new(RefCell::new(0.0));

        let mut throttler = ProgressThrottler::new(
            move |processed, total| reports_clone.borrow_mut().push((processed, total)),
            move || {
                let t = *time.borrow();
                *time.borrow_mut() += 1.0;
                t
            },
            100,    // byte threshold
            1000.0, // time threshold (not reached)
        );

        // First report: below threshold
        assert!(!throttler.maybe_report(50, 1000));
        assert_eq!(reports.borrow().len(), 0);

        // Second report: meets byte threshold
        assert!(throttler.maybe_report(150, 1000));
        assert_eq!(reports.borrow().len(), 1);
        assert_eq!(reports.borrow()[0], (150, 1000));

        // Third report: below threshold again
        assert!(!throttler.maybe_report(200, 1000));
        assert_eq!(reports.borrow().len(), 1);
    }

    #[test]
    fn throttler_reports_when_time_threshold_exceeded() {
        let reports = Rc::new(RefCell::new(Vec::new()));
        let reports_clone = reports.clone();
        let time = Rc::new(RefCell::new(0.0));

        let mut throttler = ProgressThrottler::new(
            move |processed, total| reports_clone.borrow_mut().push((processed, total)),
            move || {
                let t = *time.borrow();
                *time.borrow_mut() += 100.0;
                t
            },
            u64::MAX, // byte threshold (not reached)
            501.0,    // time threshold = 501 ms (needs >=501 to trigger)
        );

        assert!(!throttler.maybe_report(1, 1000));
        assert_eq!(reports.borrow().len(), 0);

        // Advance past time threshold
        assert!(!throttler.maybe_report(2, 1000));
        assert!(!throttler.maybe_report(3, 1000));
        assert!(!throttler.maybe_report(4, 1000));
        assert!(!throttler.maybe_report(5, 1000));
        assert!(throttler.maybe_report(6, 1000));
        assert_eq!(reports.borrow().len(), 1);
    }

    #[test]
    fn force_next_ensures_immediate_report() {
        let reports = Rc::new(RefCell::new(Vec::new()));
        let reports_clone = reports.clone();

        let mut throttler = ProgressThrottler::new(
            move |processed, total| reports_clone.borrow_mut().push((processed, total)),
            || 0.0,
            u64::MAX,
            f64::INFINITY,
        );

        assert!(!throttler.maybe_report(1, 100));
        assert_eq!(reports.borrow().len(), 0);

        throttler.force_next();
        assert!(throttler.maybe_report(1, 100));
        assert_eq!(reports.borrow().len(), 1);
        assert_eq!(reports.borrow()[0], (1, 100));
    }
}
