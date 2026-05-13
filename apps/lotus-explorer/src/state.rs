// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::app_state::AppState;
use crate::features::explore::form_actions::{FormAction, apply_form_action};
use crate::features::explore::search_state::ExploreState;
use crate::models::*;
use dioxus::prelude::*;

// ── App State Context ─────────────────────────────────────────────────────

/// Root context: access to the unified `AppState` (view, download, metrics).
///
/// Use this to read or mutate view-selection and download-orchestration state.
/// For search form state use [`FormCriteriaContext`]; for results and lifecycle
/// state use [`ResultsContext`].
#[derive(Clone, Copy)]
pub struct AppStateContext {
    pub state: Signal<AppState>,
}

impl AppStateContext {
    pub fn new(state: Signal<AppState>) -> Self {
        Self { state }
    }

    /// Mutate app state through a closure.
    ///
    /// Preferred over writing through `state` directly because it lets the
    /// call-site express *intent* rather than raw field access.
    #[allow(dead_code)] // Public extension API — not yet called from UI code
    pub fn mut_state<F>(&self, f: F)
    where
        F: FnOnce(&mut AppState),
    {
        let mut state = self.state;
        state.with_mut(f);
    }
}

// ── Form Criteria Context ─────────────────────────────────────────────────────

/// Context provider for the search form.
///
/// Provides:
/// * A single `update(FormAction)` dispatch point — all form mutations go
///   through here, eliminating per-field callback props.
/// * Dirty-state tracking via a `baseline` snapshot recorded each time the
///   user triggers a search.  Call `is_dirty()` to check whether the form
///   has been modified since the last search; call `mark_searched()` to
///   advance the baseline to the current criteria.
///
/// ## Usage
///
/// ```rust,ignore
/// let ctx = use_form_criteria_context();
/// ctx.update(FormAction::Taxon("Quercus".to_string()));
/// if ctx.is_dirty() { /* show unsaved-changes indicator */ }
/// ```
#[derive(Clone, Copy)]
pub struct FormCriteriaContext {
    /// Live search-form criteria — read/write.
    pub criteria: Signal<SearchCriteria>,
    /// Snapshot at the time of the last search — used for dirty detection.
    baseline: Signal<SearchCriteria>,
}

impl FormCriteriaContext {
    /// Create the context.  Both signals must be initialised to the same value
    /// (typically `initial_criteria_from_url()`) so that `is_dirty()` returns
    /// `false` before the user touches anything.
    pub fn new(criteria: Signal<SearchCriteria>, baseline: Signal<SearchCriteria>) -> Self {
        Self { criteria, baseline }
    }

    /// Dispatch a form action, atomically mutating the live criteria.
    pub fn update(&self, action: FormAction) {
        let mut criteria = self.criteria;
        let current = criteria.peek().clone();
        let updated = apply_form_action(current, action);
        *criteria.write() = updated;
    }

    /// `true` if the live criteria differ from the last-searched baseline.
    pub fn is_dirty(&self) -> bool {
        *self.criteria.read() != *self.baseline.read()
    }

    /// Advance the dirty baseline to the current criteria.
    ///
    /// Call this immediately before (or at the same tick as) `start_search` so
    /// that the search button returns to its non-dirty style.
    pub fn mark_searched(&self) {
        let current = self.criteria.peek().clone();
        let mut baseline = self.baseline;
        *baseline.write() = current;
    }

    /// Peek at the current criteria without subscribing to reactivity.
    #[allow(dead_code)] // Available for non-reactive inspection
    pub fn peek(&self) -> SearchCriteria {
        self.criteria.peek().clone()
    }
}

// ── Search UI Context ─────────────────────────────────────────────────────────

/// Context for the search panel.
///
/// Provides read access to the explore lifecycle signal for the search button
/// (loading flag, etc.).  Criteria are now accessed exclusively through
/// [`FormCriteriaContext`] — this context is intentionally narrowed to just
/// the explore signal to keep `SearchPanel`'s dependencies minimal.
#[derive(Clone, Copy)]
pub struct SearchUiContext {
    /// Explore lifecycle / results — read access for loading state, etc.
    pub explore: Signal<ExploreState>,
}

impl SearchUiContext {
    pub fn new(_criteria: Signal<SearchCriteria>, explore: Signal<ExploreState>) -> Self {
        Self { explore }
    }
}

// ── Results Context ───────────────────────────────────────────────────────────

/// Context for results-area components.
///
/// Contains only the live `explore` signal — previously it also held a
/// `app_state` reference that was used to read a stale mirror copy of
/// `ExploreState`.  That mirror copy has been removed; components now read the
/// live signal directly.
#[derive(Clone, Copy)]
pub struct ResultsContext {
    /// Live explore signal — results, lifecycle, UI chrome.
    pub explore: Signal<ExploreState>,
}

impl ResultsContext {
    pub fn new(explore: Signal<ExploreState>) -> Self {
        Self { explore }
    }
}

// ── Hook Helpers ──────────────────────────────────────────────────────────────

/// Hook to read the root AppStateContext from any descendant component.
#[allow(dead_code)] // Public hook — components may call this for view switching
pub fn use_app_state_context() -> AppStateContext {
    use_context::<AppStateContext>()
}

pub fn use_search_ui_context() -> SearchUiContext {
    use_context::<SearchUiContext>()
}

pub fn use_results_context() -> ResultsContext {
    use_context::<ResultsContext>()
}

pub fn use_form_criteria_context() -> FormCriteriaContext {
    use_context::<FormCriteriaContext>()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::explore::form_actions::{FormAction, apply_form_action};

    #[test]
    fn apply_form_action_taxon_round_trips() {
        let base = SearchCriteria::default();
        let updated = apply_form_action(base, FormAction::Taxon("Rosa".into()));
        assert_eq!(updated.taxon, "Rosa");
    }

    #[test]
    fn apply_form_action_mass_range_round_trips() {
        let base = SearchCriteria::default();
        let updated = apply_form_action(base, FormAction::MassMin(50.0));
        assert_eq!(updated.mass_min, 50.0);
    }

    // Dirty-tracking logic tests — exercised on the pure data model so no
    // Dioxus runtime is needed.

    #[test]
    fn form_action_all_element_bounds_round_trip() {
        use crate::models::ElementState;
        let base = SearchCriteria::default();
        let updated = apply_form_action(base.clone(), FormAction::CMin(6));
        assert_eq!(updated.c_min, 6);

        let updated = apply_form_action(base.clone(), FormAction::HMax(20));
        assert_eq!(updated.h_max, 20);

        let updated = apply_form_action(base.clone(), FormAction::FState(ElementState::Required));
        assert_eq!(updated.f_state, ElementState::Required);
    }

    #[test]
    fn form_action_formula_enabled_round_trips() {
        let base = SearchCriteria::default();
        assert!(!base.formula_enabled);
        let updated = apply_form_action(base, FormAction::FormulaEnabled(true));
        assert!(updated.formula_enabled);
    }

    #[test]
    fn form_action_smiles_search_type_round_trips() {
        use crate::models::SmilesSearchType;
        let base = SearchCriteria::default();
        let updated = apply_form_action(
            base,
            FormAction::SmilesSearchType(SmilesSearchType::Similarity),
        );
        assert_eq!(updated.smiles_search_type, SmilesSearchType::Similarity);
    }

    #[test]
    fn form_action_year_range_round_trips() {
        let base = SearchCriteria::default();
        let updated = apply_form_action(base.clone(), FormAction::YearMin(1990));
        assert_eq!(updated.year_min, 1990);
        let updated = apply_form_action(base, FormAction::YearMax(2025));
        assert_eq!(updated.year_max, 2025);
    }
}
