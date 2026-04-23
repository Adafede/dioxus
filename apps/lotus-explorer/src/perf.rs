//! Performance monitoring and logging infrastructure for SPARQL query execution.
//!
//! Provides cross-platform logging (WASM console vs. native stdout) and fine-grained
//! timing measurements across all query phases.

use std::collections::BTreeMap;
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

/// Timing data for a single query execution phase.
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
pub struct PhaseTimer {
    pub phase: String,
    pub started_at: f64,
    pub duration: Option<Duration>,
}

#[cfg(target_arch = "wasm32")]
impl PhaseTimer {
    pub fn new(phase: &str) -> Self {
        Self {
            phase: phase.to_string(),
            started_at: wasm_now_ms(),
            duration: None,
        }
    }

    pub fn end(&mut self) -> Duration {
        let ms = (wasm_now_ms() - self.started_at).max(0.0);
        let d = Duration::from_secs_f64(ms / 1000.0);
        self.duration = Some(d);
        d
    }
}

#[cfg(target_arch = "wasm32")]
fn wasm_now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug)]
pub struct PhaseTimer {
    pub phase: String,
    pub started_at: Instant,
    pub duration: Option<Duration>,
}

#[cfg(not(target_arch = "wasm32"))]
impl PhaseTimer {
    pub fn new(phase: &str) -> Self {
        Self {
            phase: phase.to_string(),
            started_at: Instant::now(),
            duration: None,
        }
    }

    pub fn end(&mut self) -> Duration {
        let d = self.started_at.elapsed();
        self.duration = Some(d);
        d
    }
}

/// Aggregated timing metrics for an entire search operation.
#[derive(Clone, Debug, Default)]
pub struct QueryMetrics {
    pub total_duration: Option<Duration>,
    pub phases: BTreeMap<String, Duration>,
    pub query_size_bytes: usize,
    pub response_size_bytes: usize,
    pub parsed_rows: usize,
    pub deduped_rows: usize,
}

impl QueryMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_phase(&mut self, phase: &str, duration: Duration) {
        self.phases.insert(phase.to_string(), duration);
    }

    pub fn record_total(&mut self, duration: Duration) {
        self.total_duration = Some(duration);
    }

    /// Convert to JSON for embedding in metadata.
    pub fn to_json(&self) -> serde_json::Value {
        let mut obj = serde_json::json!({});

        if let Some(total) = self.total_duration {
            obj["total_ms"] = serde_json::json!(total.as_millis() as u64);
        }

        if !self.phases.is_empty() {
            let phases_obj = self
                .phases
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::json!(v.as_millis() as u64)))
                .collect::<serde_json::Map<String, serde_json::Value>>();
            obj["phases_ms"] = serde_json::Value::Object(phases_obj);
        }

        if self.query_size_bytes > 0 {
            obj["query_size_bytes"] = serde_json::json!(self.query_size_bytes);
        }
        if self.response_size_bytes > 0 {
            obj["response_size_bytes"] = serde_json::json!(self.response_size_bytes);
        }
        if self.parsed_rows > 0 {
            obj["parsed_rows"] = serde_json::json!(self.parsed_rows);
        }
        if self.deduped_rows > 0 {
            obj["deduped_rows"] = serde_json::json!(self.deduped_rows);
        }

        obj
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_to_json_includes_all_fields() {
        let mut m = QueryMetrics::new();
        m.record_total(Duration::from_millis(500));
        m.record_phase("Counting", Duration::from_millis(100));
        m.record_phase("FetchingPreview", Duration::from_millis(300));
        m.query_size_bytes = 1024;
        m.response_size_bytes = 102400;
        m.parsed_rows = 1000;

        let json = m.to_json();
        assert_eq!(json["total_ms"], 500);
        assert_eq!(json["phases_ms"]["Counting"], 100);
        assert_eq!(json["query_size_bytes"], 1024);
        assert_eq!(json["response_size_bytes"], 102400);
    }
}
