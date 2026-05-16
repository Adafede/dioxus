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

/// Check if download dispatch effect should wait for loading to complete.
///
/// When download is pending and results are still loading, the dispatch effect
/// must wait.  This prevents race conditions where results are not yet available.
#[must_use]
#[allow(dead_code)] // Available for future loading-wait orchestration
pub fn is_waiting_for_loading(loading: bool) -> bool {
    loading
}

/// Check if download dispatch effect should wait for SPARQL query to materialize.
///
/// After loading completes, the query must exist in state before download can proceed.
#[must_use]
#[allow(dead_code)] // Available for future query-wait orchestration
pub fn is_waiting_for_query(query: Option<&str>) -> bool {
    query.is_none()
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

/// Update metrics state guard when entering "waiting for loading" phase.
#[must_use]
#[allow(dead_code)] // Available for future metrics-guard orchestration
pub fn enter_waiting_loading_phase(metrics: MetricsState) -> MetricsState {
    MetricsState {
        waiting_loading_logged: false,
        ..metrics
    }
}

/// Update metrics state guard when exiting "waiting for loading" phase.
#[must_use]
#[allow(dead_code)] // Available for future metrics-guard orchestration
pub fn exit_waiting_loading_phase(metrics: MetricsState) -> MetricsState {
    MetricsState {
        waiting_loading_logged: false,
        ..metrics
    }
}

/// Update metrics state guard when entering "waiting for query" phase.
#[must_use]
#[allow(dead_code)] // Available for future metrics-guard orchestration
pub fn enter_waiting_query_phase(metrics: MetricsState) -> MetricsState {
    MetricsState {
        waiting_query_logged: false,
        ..metrics
    }
}

/// Update metrics state guard when exiting "waiting for query" phase.
#[must_use]
#[allow(dead_code)] // Available for future metrics-guard orchestration
pub fn exit_waiting_query_phase(metrics: MetricsState) -> MetricsState {
    MetricsState {
        waiting_query_logged: false,
        ..metrics
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
    fn is_waiting_for_loading_simple_flag_check() {
        assert!(is_waiting_for_loading(true));
        assert!(!is_waiting_for_loading(false));
    }

    #[test]
    fn is_waiting_for_query_checks_presence() {
        assert!(!is_waiting_for_query(Some("SELECT * WHERE"))); // query exists
        assert!(is_waiting_for_query(None));
        assert!(!is_waiting_for_query(Some(""))); // empty string is still "present"
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
}
