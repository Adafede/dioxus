// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Consolidated app state architecture.
//!
//! Instead of 8 independent signals at app root, AppState groups related state
//! into focused sub-structures with clear ownership and lifecycle.

use crate::app::view::AppView;
use crate::features::explore::search_state::ExploreState;
use crate::i18n::Locale;
use crate::models::SearchCriteria;

/// Root application state — single source of truth for all app-level signals.
///
/// Replaces:
/// - app_view: Signal<AppView>
/// - criteria: Signal<SearchCriteria>
/// - locale: Signal<Locale>
/// - explore: Signal<ExploreState>
/// - pending_download_format: Signal<Option<String>>
/// - pending_execute: Signal<bool>
/// - waiting_loading_logged: Signal<bool>
/// - waiting_query_logged: Signal<bool>
///
/// With:
/// - Single structured AppState signal at app root
/// - Clear ownership and relationships between sub-structures
/// - Easier to reason about state consistency
#[allow(dead_code)] // Will be fully integrated when main.rs is refactored
#[derive(Clone, PartialEq)]
pub struct AppState {
    /// View selection (Explore vs Draw vs Curation)
    pub view: AppView,

    /// Search form and results — grouped coherently
    pub search: SearchState,

    /// UI chrome (locale, mobile filters, etc.)
    pub ui: UiState,

    /// Download orchestration state
    pub download: DownloadState,

    /// Performance and telemetry tracking
    pub metrics: MetricsState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            view: AppView::Explore,
            search: SearchState::default(),
            ui: UiState::default(),
            download: DownloadState::default(),
            metrics: MetricsState::default(),
        }
    }
}

// ── Search State: Form + Results ───────────────────────────────────────

/// Search form and result state — the primary concern of the application.
#[allow(dead_code)] // Will be integrated in Phase 2 main.rs refactoring
#[derive(Clone, PartialEq, Default)]
pub struct SearchState {
    /// Current form criteria
    pub criteria: SearchCriteria,

    /// Explore feature state (results, loading, errors, etc.)
    pub explore: ExploreState,
}

// ── UI State: Chrome and Navigation ────────────────────────────────────

/// UI chrome, locale, and navigation state.
#[allow(dead_code)] // Will be integrated in Phase 2 main.rs refactoring
#[derive(Clone, PartialEq)]
pub struct UiState {
    /// Current locale for i18n
    pub locale: Locale,

    /// Whether mobile filter panel is open
    pub mobile_filters_open: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            locale: Locale::En,
            mobile_filters_open: false,
        }
    }
}

// ── Download State: Orchestration ──────────────────────────────────────

/// Download action orchestration and state.
#[allow(dead_code)] // Will be integrated in Phase 2 main.rs refactoring
#[derive(Clone, PartialEq, Default)]
pub struct DownloadState {
    /// Pending download format (if any)
    pub pending_format: Option<String>,

    /// Whether direct download mode is active
    pub direct_execute: bool,
}

// ── Metrics State: Telemetry ───────────────────────────────────────────

/// Performance tracking and telemetry state.
#[allow(dead_code)] // Will be integrated in Phase 2 main.rs refactoring
#[derive(Clone, PartialEq, Default)]
pub struct MetricsState {
    /// Whether we've already logged the "waiting for load" metric
    pub waiting_loading_logged: bool,

    /// Whether we've already logged the "waiting for query" metric
    pub waiting_query_logged: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_state_default_creates_valid_initial_state() {
        let state = AppState::default();
        assert_eq!(state.view, AppView::Explore);
        assert!(!state.ui.mobile_filters_open);
        assert_eq!(state.ui.locale, Locale::En);
        assert!(!state.metrics.waiting_loading_logged);
    }

    #[test]
    fn search_state_default_has_empty_criteria() {
        let state = SearchState::default();
        assert_eq!(state.criteria, SearchCriteria::default());
    }

    #[test]
    fn ui_state_default_is_sensible() {
        let state = UiState::default();
        assert_eq!(state.locale, Locale::En);
        assert!(!state.mobile_filters_open);
    }

    #[test]
    fn download_state_default_is_inactive() {
        let state = DownloadState::default();
        assert!(state.pending_format.is_none());
        assert!(!state.direct_execute);
    }
}
