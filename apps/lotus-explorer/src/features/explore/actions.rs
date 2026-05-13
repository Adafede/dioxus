// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

#![allow(dead_code)]

//! Typed action catalog for the Explore feature's state machine.
//!
//! # Current role
//!
//! `ExploreAction` is an **enum-as-documentation** that names every possible
//! state transition in the explore search lifecycle.  At present the
//! application still uses 19 independent Dioxus `Signal<T>` values (via
//! `SearchRuntime`) rather than a single reducer-driven store, because
//! independent signals give fine-grained reactivity: e.g., a `query_phase`
//! change only re-renders the `LoadingState` spinner, not the entire
//! `ResultsViewport` tree.
//!
//! # Migration path
//!
//! If the signal count grows further or state consistency bugs emerge,
//! convert to a full reducer:
//!
//! ```rust,ignore
//! // 1. Define a flat ExploreState struct.
//! // 2. Replace 19 signals with one Signal<ExploreState>.
//! // 3. Replace set_signal_if_changed() calls with dispatch(action).
//! // 4. Write a pure reduce(state, action) -> ExploreState function.
//! // 5. Test reduce() in standard unit tests — no Dioxus context needed.
//! ```
//!
//! The tradeoff is coarser reactivity (any state change re-renders every
//! subscriber of `Signal<ExploreState>`) vs simpler correctness guarantees.
//!
//! # Testability today
//!
//! Because `orchestrator::do_search` is already a pure async function that
//! returns `SearchOutcome` without touching any signal, it can be unit-tested
//! with a `MockRepository` (see [`crate::repositories`]) without wiring up
//! the full Dioxus runtime.

use crate::features::explore::types::{AppError, QueryPhase};
use crate::models::{DatasetStats, SearchCriteria};

/// Every distinct state transition that can occur in the Explore search
/// lifecycle.  Names serve as living documentation of the state machine.
#[allow(dead_code)]
pub enum ExploreAction {
    // ── Search lifecycle ──────────────────────────────────────────────────

    /// User (or programmatic startup) triggered a new search.
    /// Carries the criteria snapshot and whether this is a silent download-only
    /// mode that suppresses preview rendering.
    SearchTriggered {
        criteria: SearchCriteria,
        direct_download: bool,
    },

    /// The in-flight search phase advanced to a new step.
    PhaseAdvanced(QueryPhase),

    /// The search completed successfully.
    SearchSucceeded {
        qid: Option<String>,
        warning: Option<String>,
        query: String,
        rows_count: usize,
        total_matches: Option<usize>,
        total_stats: Option<DatasetStats>,
        query_hash: String,
        result_hash: String,
        display_capped: bool,
    },

    /// The search failed with a typed error.
    SearchFailed(AppError),

    /// A superseded in-flight request completed; its result was discarded.
    StaleResultDiscarded { request_token: u64 },

    // ── Error handling ────────────────────────────────────────────────────

    /// The user dismissed the current error notice.
    ErrorDismissed,

    /// The user pressed "Retry" on the error notice.
    RetryRequested,

    // ── Download ──────────────────────────────────────────────────────────

    /// URL contained `download=true&format=…`; download was auto-triggered on startup.
    StartupDownloadTriggered { format: String },

    /// URL contained `execute=true`; search was auto-triggered on startup.
    StartupExecuteTriggered,

    /// The SPARQL query became available; file download was dispatched.
    DownloadDispatched { format: String },

    /// The file download completed (success or error — both clear dispatching flag).
    DownloadCompleted,

    // ── UI / navigation ───────────────────────────────────────────────────

    /// The user switched to a different view tab.
    ViewChanged(crate::app::view::AppView),

    /// The user switched the active locale.
    LocaleChanged(crate::i18n::Locale),

    /// The user opened/closed the mobile filter drawer.
    MobileFiltersToggled,
}
