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
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Reports every `(processed, total)` the throttler emits.
    type Sink = Box<dyn FnMut(u64, u64)>;
    /// A clock, in milliseconds.
    type Clock = Box<dyn Fn() -> f64>;

    /// A throttler wired to a test-controlled clock, and the reports it made.
    ///
    /// `Box`ing the two closures keeps one recorder shape for every case below,
    /// so a test only states the two thresholds and the clock step.
    struct Recorder {
        throttler: ProgressThrottler<Sink, Clock>,
        reports: Rc<RefCell<Vec<(u64, u64)>>>,
    }

    impl Recorder {
        fn new(byte_threshold: u64, time_threshold_ms: f64, step_ms: f64) -> Self {
            let reports = Rc::new(RefCell::new(Vec::new()));
            let time = Rc::new(RefCell::new(0.0));
            let sink = reports.clone();
            let clock = move || {
                let now = *time.borrow();
                *time.borrow_mut() += step_ms;
                now
            };
            let throttler = ProgressThrottler::new(
                Box::new(move |processed, total| sink.borrow_mut().push((processed, total)))
                    as Sink,
                Box::new(clock) as Clock,
                byte_threshold,
                time_threshold_ms,
            );
            Self { throttler, reports }
        }
    }

    #[test]
    fn throttler_reports_when_byte_threshold_exceeded() {
        let mut r = Recorder::new(100, 1000.0, 1.0);

        assert!(
            !r.throttler.maybe_report(50, 1000),
            "50 of a 100-byte threshold must not report"
        );
        assert!(r.reports.borrow().is_empty(), "nothing was reported yet");

        assert!(
            r.throttler.maybe_report(150, 1000),
            "150 of a 100-byte threshold must report"
        );
        assert_eq!(
            *r.reports.borrow(),
            [(150, 1000)],
            "the crossing call is the one reported"
        );

        assert!(
            !r.throttler.maybe_report(200, 1000),
            "50 bytes past the last report is below the threshold again"
        );
        assert_eq!(r.reports.borrow().len(), 1, "the throttler did not reset");
    }

    #[test]
    fn throttler_reports_when_time_threshold_exceeded() {
        // A byte threshold no upload reaches, and a clock that jumps 100 ms per
        // call, so only elapsed time can trigger a report.
        let mut r = Recorder::new(u64::MAX, 501.0, 100.0);

        for processed in 1..=5 {
            assert!(
                !r.throttler.maybe_report(processed, 1000),
                "{processed} calls of 100 ms have not reached 501 ms"
            );
        }
        assert!(
            r.throttler.maybe_report(6, 1000),
            "the sixth call reaches 600 ms, past the 501 ms threshold"
        );
        assert_eq!(r.reports.borrow().len(), 1, "exactly one report so far");
    }

    #[test]
    fn a_zero_byte_threshold_reports_before_anything_is_read() {
        // A caller that has read nothing still has to paint "0 / total", and a
        // zero byte threshold is how it asks for that: `0 >= 0` is the only
        // report available before any bytes exist, so the byte arm has to be
        // what fires rather than the time arm.
        let mut r = Recorder::new(0, f64::INFINITY, 0.0);

        assert!(
            r.throttler.maybe_report(0, 100),
            "a zero byte threshold reports with nothing read"
        );
        assert_eq!(*r.reports.borrow(), [(0, 100)], "the zero-byte report");
    }

    #[test]
    fn an_unreachable_threshold_never_reports() {
        // The configuration the two above are the mirror of: no byte threshold
        // is reachable and the clock never moves. Nothing is reported, which is
        // what keeps a stalled read from repainting the UI forever.
        let mut r = Recorder::new(u64::MAX, f64::INFINITY, 0.0);

        assert!(
            !r.throttler.maybe_report(1, 100),
            "an unreachable threshold must not report"
        );
        assert!(r.reports.borrow().is_empty(), "and must not call back");
    }
}
