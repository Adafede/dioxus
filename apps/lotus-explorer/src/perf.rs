//! Performance monitoring and logging infrastructure for SPARQL query execution.
//!
//! Provides cross-platform logging (WASM console vs. native stdout) and fine-grained
//! timing measurements across all query phases.

use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

/// Timer handle - stores platform-specific timing data.
/// On WASM, this stores `performance.now()` milliseconds.
/// On native, this stores the Instant when started
#[cfg(target_arch = "wasm32")]
pub type TimerHandle = f64;

#[cfg(not(target_arch = "wasm32"))]
pub type TimerHandle = Instant;

#[cfg(target_arch = "wasm32")]
fn wasm_now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}


/// Log a message with timing context. Works cross-platform (WASM console vs. native stdout).
pub fn log_timing(phase: &str, message: &str, duration: Option<Duration>) {
    let msg = if let Some(d) = duration {
        format!(
            "[LOTUS:{}] {} ({:.1}ms)",
            phase,
            message,
            d.as_secs_f64() * 1000.0
        )
    } else {
        format!("[LOTUS:{}] {}", phase, message)
    };

    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::info_1(&msg.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        log::info!("{}", msg);
    }
}

/// Log a message at info level. Works cross-platform.
pub fn log_info(message: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::info_1(&message.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        log::info!("{}", message);
    }
}

/// Log a message at debug level. Works cross-platform.
pub fn log_debug(message: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::debug_1(&message.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        log::debug!("{}", message);
    }
}

/// Log a message at warn level. Works cross-platform.
pub fn log_warn(message: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::warn_1(&message.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        log::warn!("{}", message);
    }
}

/// Start a console.time() block on WASM, or return the current timestamp on native.
#[cfg(target_arch = "wasm32")]
pub fn start_timer(_label: &str) -> TimerHandle {
    web_sys::console::time_with_label(_label);
    wasm_now_ms()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn start_timer(_label: &str) -> TimerHandle {
    Instant::now()
}

/// End a console.time() block on WASM and compute elapsed duration on native.
#[cfg(target_arch = "wasm32")]
pub fn end_timer(_label: &str, started: TimerHandle) -> Duration {
    web_sys::console::time_end_with_label(_label);
    let elapsed_ms = (wasm_now_ms() - started).max(0.0);
    Duration::from_secs_f64(elapsed_ms / 1000.0)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn end_timer(_label: &str, started: TimerHandle) -> Duration {
    started.elapsed()
}

// ...existing code...
