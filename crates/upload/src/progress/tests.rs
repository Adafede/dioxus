// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

// The `tests` tests, extracted from `progress.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `progress` module, so
// `use super::*` below reaches exactly what it did before the move.
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
            Box::new(move |processed, total| sink.borrow_mut().push((processed, total))) as Sink,
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
