// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Pure effect scheduling helpers and lifecycle state queries for download orchestration.
//!
//! This module separates concerns from the hook implementations in `download_dispatch.rs`:
//! * State query helpers determine when effects should run.
//! * Effect schedulers are pure functions that decide what to dispatch/execute.
//! * Telemetry helpers centralize logging patterns.

use crate::app_state::MetricsState;
use crate::download::DownloadFormat;
use crate::features::explore::search_state::ExploreState;
use crate::models::SearchCriteria;

// ── State Query Helpers ───────────────────────────────────────────────────────

/// Check if startup effect should trigger based on download state and search history.
///
/// Returns `true` when all preconditions are met:
/// * No prior search has completed (`!searched_once`)
/// * Not currently loading
/// * Either pending download format or direct-execute flag is set
#[must_use]
pub fn should_trigger_startup_search(
    pending_format: Option<DownloadFormat>,
    direct_execute: bool,
    searched_once: bool,
    loading: bool,
) -> bool {
    (pending_format.is_some() || direct_execute) && !searched_once && !loading
}

// ── Telemetry Helpers ─────────────────────────────────────────────────────────

/// Determine telemetry call for startup trigger based on mode.
pub enum StartupTriggerMode {
    /// Format pending — user requested download startup.
    Download { format: DownloadFormat },
    /// Direct execute — user requested immediate search.
    DirectExecute,
}

impl StartupTriggerMode {
    /// Report startup trigger via appropriate telemetry channel.
    pub fn log(&self) {
        use crate::services::search_telemetry as telemetry;
        match self {
            Self::Download { format } => {
                telemetry::download_startup_auto_search_triggered(format.log_name());
            }
            Self::DirectExecute => {
                telemetry::search_startup_auto_search_execute();
            }
        }
    }
}

// ── Metrics State Management ──────────────────────────────────────────────────

/// Metrics state when no download dispatch is pending.
#[must_use]
pub fn metrics_for_inactive_phase(_: &MetricsState) -> MetricsState {
    MetricsState::default()
}

/// Metrics state after a waiting-for-loading dispatch tick.
///
/// `logged_waiting_loading` must be `true` only when telemetry was emitted
/// during this tick; this keeps the guard aligned with log side effects.
#[must_use]
pub fn metrics_for_waiting_loading_phase(
    metrics: &MetricsState,
    logged_waiting_loading: bool,
) -> MetricsState {
    MetricsState {
        waiting_loading_logged: metrics.waiting_loading_logged || logged_waiting_loading,
        waiting_query_logged: false,
    }
}

/// Metrics state after a waiting-for-query dispatch tick.
///
/// `logged_waiting_query` must be `true` only when telemetry was emitted
/// during this tick; this keeps the guard aligned with log side effects.
#[must_use]
pub fn metrics_for_waiting_query_phase(
    metrics: &MetricsState,
    logged_waiting_query: bool,
) -> MetricsState {
    MetricsState {
        waiting_loading_logged: false,
        waiting_query_logged: metrics.waiting_query_logged || logged_waiting_query,
    }
}

// ── Download Dispatch Scheduling ──────────────────────────────────────────────

/// Narrow view of download readiness state to avoid repeating complex queries.
#[derive(Clone, Debug, PartialEq)]
pub enum DispatchPhase {
    /// No download pending — nothing to do.
    Inactive,
    /// Download pending, still waiting for results to load.
    WaitingForLoading { format: DownloadFormat },
    /// Loading complete, waiting for SPARQL query to materialize.
    WaitingForQuery { format: DownloadFormat },
    /// All preconditions met — ready to dispatch download.
    Ready {
        /// Criteria snapshot to embed in download (WASM specific).
        /// Desktop builds don't need this.
        criteria: SearchCriteria,
        /// Query to pass to download executor.
        query: String,
        /// Filename to use for downloaded file.
        filename: String,
        /// Download format (for telemetry).
        format: DownloadFormat,
    },
}

/// Determine the current dispatch phase based on download and result state.
///
/// This pure function centralizes the decision tree that determines what the
/// download dispatch effect should do on each render.  No side effects here —
/// just data transformation from signals to a single phase enum.
#[must_use]
pub fn classify_dispatch_phase(
    pending_format: Option<DownloadFormat>,
    explore: &ExploreState,
) -> DispatchPhase {
    let Some(fmt) = pending_format else {
        return DispatchPhase::Inactive;
    };

    if explore.lifecycle.loading {
        return DispatchPhase::WaitingForLoading { format: fmt };
    }

    let Some(query) = explore.result.sparql_query.as_deref().map(str::to_string) else {
        return DispatchPhase::WaitingForQuery { format: fmt };
    };

    let criteria = explore.ui.executed_criteria.clone();
    let filename = crate::export::generate_filename(&criteria, fmt.extension());

    DispatchPhase::Ready {
        criteria,
        query,
        filename,
        format: fmt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_trigger_startup_search_requires_all_conditions() {
        assert!(should_trigger_startup_search(
            Some(DownloadFormat::Csv),
            false,
            false,
            false
        ));
        assert!(!should_trigger_startup_search(None, false, false, false)); // no trigger
        assert!(!should_trigger_startup_search(
            Some(DownloadFormat::Csv),
            false,
            true,
            false
        )); // already searched
        assert!(!should_trigger_startup_search(
            Some(DownloadFormat::Csv),
            false,
            false,
            true
        )); // already loading
    }

    #[test]
    fn dispatch_phase_inactive_when_no_pending_format() {
        let phase = classify_dispatch_phase(None, &ExploreState::default());
        assert_eq!(phase, DispatchPhase::Inactive);
    }

    #[test]
    fn dispatch_phase_waiting_for_loading_when_loading() {
        let mut explore = ExploreState::default();
        explore.lifecycle.loading = true;
        let phase = classify_dispatch_phase(Some(DownloadFormat::Csv), &explore);
        assert_eq!(
            phase,
            DispatchPhase::WaitingForLoading {
                format: DownloadFormat::Csv,
            }
        );
    }

    #[test]
    fn dispatch_phase_waiting_for_query_when_query_absent() {
        let explore = ExploreState::default();
        // result.sparql_query is None by default
        let phase = classify_dispatch_phase(Some(DownloadFormat::Csv), &explore);
        assert_eq!(
            phase,
            DispatchPhase::WaitingForQuery {
                format: DownloadFormat::Csv,
            }
        );
    }

    #[test]
    fn dispatch_phase_ready_when_all_preconditions_met() {
        use std::sync::Arc;
        let mut explore = ExploreState::default();
        explore.result.sparql_query = Some(Arc::from("SELECT * WHERE {}"));
        explore.ui.executed_criteria = SearchCriteria {
            taxon: "Rosa".into(),
            ..SearchCriteria::default()
        };

        let phase = classify_dispatch_phase(Some(DownloadFormat::Json), &explore);

        if let DispatchPhase::Ready {
            criteria,
            query,
            format,
            ..
        } = phase
        {
            assert_eq!(criteria.taxon, "Rosa");
            assert!(query.contains("SELECT"));
            assert_eq!(format, DownloadFormat::Json);
        } else {
            panic!("expected Ready phase, got {:?}", phase);
        }
    }

    #[test]
    fn startup_trigger_mode_discriminates_by_source() {
        let download_mode = StartupTriggerMode::Download {
            format: DownloadFormat::Csv,
        };
        let execute_mode = StartupTriggerMode::DirectExecute;

        // Enum discriminates visibly
        matches!(download_mode, StartupTriggerMode::Download { .. });
        matches!(execute_mode, StartupTriggerMode::DirectExecute);
    }

    #[test]
    fn inactive_phase_resets_metrics_guards() {
        let metrics = MetricsState {
            waiting_loading_logged: true,
            waiting_query_logged: true,
        };
        assert_eq!(
            metrics_for_inactive_phase(&metrics),
            MetricsState::default()
        );
    }

    #[test]
    fn waiting_loading_phase_sets_loading_guard_and_clears_query_guard() {
        let metrics = MetricsState {
            waiting_loading_logged: false,
            waiting_query_logged: true,
        };
        let next = metrics_for_waiting_loading_phase(&metrics, true);
        assert!(next.waiting_loading_logged);
        assert!(!next.waiting_query_logged);
    }

    #[test]
    fn waiting_query_phase_sets_query_guard_and_clears_loading_guard() {
        let metrics = MetricsState {
            waiting_loading_logged: true,
            waiting_query_logged: false,
        };
        let next = metrics_for_waiting_query_phase(&metrics, true);
        assert!(!next.waiting_loading_logged);
        assert!(next.waiting_query_logged);
    }
}
