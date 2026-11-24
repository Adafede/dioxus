// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::*;
use std::cell::RefCell;
use std::rc::Rc;

type Sink = Box<dyn FnMut(u64, u64)>;
type Clock = Box<dyn Fn() -> f64>;

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
    let mut r = Recorder::new(0, f64::INFINITY, 0.0);

    assert!(
        r.throttler.maybe_report(0, 100),
        "a zero byte threshold reports with nothing read"
    );
    assert_eq!(*r.reports.borrow(), [(0, 100)], "the zero-byte report");
}

#[test]
fn an_unreachable_threshold_never_reports() {
    let mut r = Recorder::new(u64::MAX, f64::INFINITY, 0.0);

    assert!(
        !r.throttler.maybe_report(1, 100),
        "an unreachable threshold must not report"
    );
    assert!(r.reports.borrow().is_empty(), "and must not call back");
}

#[test]
fn recovers_after_clock_rollback() {
    let reports = Rc::new(RefCell::new(Vec::new()));
    let time = Rc::new(RefCell::new(500.0));
    let mut throttler = {
        let reports = Rc::clone(&reports);
        let time = Rc::clone(&time);
        ProgressThrottler::new(
            move |p, t| reports.borrow_mut().push((p, t)),
            move || *time.borrow(),
            u64::MAX,
            100.0,
        )
    };

    *time.borrow_mut() = 100.0;
    assert!(
        !throttler.maybe_report(10, 1000),
        "rollback check should reset baseline without spuriously firing at delta 0"
    );

    *time.borrow_mut() = 200.0;
    assert!(
        throttler.maybe_report(20, 1000),
        "throttler must recover within time_threshold_ms after clock rollback"
    );
    assert_eq!(*reports.borrow(), vec![(20, 1000)]);
}
