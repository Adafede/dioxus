// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

pub fn log_info_evt(event: &str, phase: &str, state: &str, details: Option<&str>) {
    let msg = match details {
        Some(d) if !d.is_empty() => format!("event={event} phase={phase} state={state} {d}"),
        _ => format!("event={event} phase={phase} state={state}"),
    };
    crate::perf::log_info(&msg);
}

pub fn log_debug_evt(event: &str, phase: &str, state: &str, details: Option<&str>) {
    let msg = match details {
        Some(d) if !d.is_empty() => format!("event={event} phase={phase} state={state} {d}"),
        _ => format!("event={event} phase={phase} state={state}"),
    };
    crate::perf::log_debug(&msg);
}

pub fn log_warn_evt(event: &str, phase: &str, state: &str, details: Option<&str>) {
    let msg = match details {
        Some(d) if !d.is_empty() => format!("event={event} phase={phase} state={state} {d}"),
        _ => format!("event={event} phase={phase} state={state}"),
    };
    crate::perf::log_warn(&msg);
}

pub fn log_timing_evt(
    event: &str,
    phase: &str,
    state: &str,
    duration: std::time::Duration,
    details: Option<&str>,
) {
    let elapsed_ms = duration.as_secs_f64() * 1000.0;
    let msg = match details {
        Some(d) if !d.is_empty() => {
            format!("event={event} phase={phase} state={state} elapsed_ms={elapsed_ms:.1} {d}")
        }
        _ => format!("event={event} phase={phase} state={state} elapsed_ms={elapsed_ms:.1}"),
    };
    crate::perf::log_info(&msg);
}
