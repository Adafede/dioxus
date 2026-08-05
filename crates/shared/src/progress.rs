// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

#![allow(clippy::module_name_repetitions)]

/// Throttles callbacks based on bytes processed and wall-clock time elapsed.
///
/// Useful for streaming operations (file reading, network downloads, parsing)
/// where you want progress updates without flooding the callback with too many
/// invocations. Implements both byte-threshold and time-threshold throttling.
///
/// # Generic Arguments
/// - `F`: Callback function signature, typically `FnMut(u64, u64)` for `(processed, total)`
/// - `T`: Time source returning milliseconds as f64 (e.g., `js_sys::Date::now` for WASM)
///
/// # Example (WASM)
/// ```ignore
/// let mut reporter = ProgressThrottler::new(
///     |processed, total| eprintln!("{}/{}", processed, total),
///     js_sys::Date::now,
///     4 * 1024 * 1024, // report every 4 MiB
///     120.0,            // or every 120ms
/// );
/// reporter.maybe_report(bytes_read, total_size); // returns true if callback fired
/// ```
#[allow(missing_debug_implementations)]
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
    /// - `callback`: Called with `(bytes_processed, total_bytes)` when thresholds met
    /// - `time_fn`: Returns current time in milliseconds (e.g., `js_sys::Date::now`)
    /// - `byte_threshold`: Report after processing at least this many bytes
    /// - `time_threshold_ms`: Report after at least this many milliseconds
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

    /// Returns `true` and calls the callback if enough bytes or time has elapsed.
    ///
    /// Tracks `processed` bytes and wall-clock time; if either threshold is exceeded
    /// since the last report, invokes the callback with `(processed, total)` and
    /// returns `true`. Otherwise, returns `false` without calling the callback.
    ///
    /// # Complexity
    /// O(1): Single comparison and possible callback invocation.
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

    /// Forces the next call to `maybe_report` to report immediately,
    /// regardless of thresholds (useful for flushing at end of stream).
    #[allow(clippy::missing_const_for_fn)]
    pub fn force_next(&mut self) {
        self.last_reported_bytes = u64::MAX;
        self.last_reported_time = f64::NEG_INFINITY;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn throttler_reports_when_byte_threshold_exceeded() {
        let reports = Rc::new(RefCell::new(Vec::new()));
        #[allow(clippy::redundant_clone)]
        let reports_clone = reports.clone();
        let time = Rc::new(RefCell::new(0.0));
        #[allow(clippy::redundant_clone)]
        let time_clone = time.clone();

        let mut throttler = ProgressThrottler::new(
            move |processed, total| {
                reports_clone.borrow_mut().push((processed, total));
            },
            move || {
                let t = *time_clone.borrow();
                *time_clone.borrow_mut() += 1.0;
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
        #[allow(clippy::redundant_clone)]
        let reports_clone = reports.clone();
        let time = Rc::new(RefCell::new(0.0));
        #[allow(clippy::redundant_clone)]
        let time_clone = time.clone();

        let mut throttler = ProgressThrottler::new(
            move |processed, total| {
                reports_clone.borrow_mut().push((processed, total));
            },
            move || {
                let t = *time_clone.borrow();
                *time_clone.borrow_mut() += 100.0; // Advance 100ms per call
                t
            },
            u64::MAX, // byte threshold (not reached)
            500.0,    // time threshold = 500ms
        );

        // First report: below both thresholds
        assert!(!throttler.maybe_report(10, 1000));
        assert_eq!(reports.borrow().len(), 0);

        // Second report: time threshold crossed (advanced 100ms)
        assert!(!throttler.maybe_report(20, 1000));
        assert_eq!(reports.borrow().len(), 0);

        // Third report: time threshold crossed again (200ms total)
        assert!(!throttler.maybe_report(30, 1000));
        assert_eq!(reports.borrow().len(), 0);

        // Fourth report: exceeds 500ms threshold
        assert!(!throttler.maybe_report(40, 1000));
        assert_eq!(reports.borrow().len(), 0);

        // Fifth report: now exceeds time threshold
        assert!(throttler.maybe_report(50, 1000));
        assert_eq!(reports.borrow().len(), 1);
        assert_eq!(reports.borrow()[0], (50, 1000));
    }

    #[test]
    fn force_next_ensures_immediate_report() {
        let reports = Rc::new(RefCell::new(Vec::new()));
        let reports_clone = reports.clone();

        let mut throttler = ProgressThrottler::new(
            move |processed, total| {
                reports_clone.borrow_mut().push((processed, total));
            },
            || 0.0,
            u64::MAX,      // impossible byte threshold
            f64::INFINITY, // impossible time threshold
        );

        // Normal report should be skipped
        assert!(!throttler.maybe_report(1, 100));
        assert_eq!(reports.borrow().len(), 0);

        // Force next and report should fire
        throttler.force_next();
        assert!(throttler.maybe_report(1, 100));
        assert_eq!(reports.borrow().len(), 1);
        assert_eq!(reports.borrow()[0], (1, 100));
    }
}
