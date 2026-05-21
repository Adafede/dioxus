// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! `use_memo`-based derived selectors for [`ExploreState`].
//!
//! Components that only care about a sub-set of state can subscribe via one
//! of these selectors instead of reading the whole `Signal<ExploreState>`.
//! The returned `Memo<T>` only fires a re-render when the **selected value**
//! changes, not on every `ExploreState` mutation.
//!
//! ## Example
//!
//! ```rust,ignore
//! // Component only re-renders when `loading` changes, not on sort changes.
//! let loading = use_lifecycle_selector(explore, |lc| lc.loading);
//! ```
//!
//! ## Wired consumers
//! * [`crate::components::results_viewport::ResultsViewport`] — uses
//!   `use_lifecycle_selector` and `use_result_selector`
//! * [`crate::components::results_table::ResultsTable`] — uses `use_result_selector`
//! * [`crate::components::results_table::toolbar::ResultsToolbar`] — uses both
//! * [`crate::components::layout::sidebar::Sidebar`] — uses `use_ui_selector`
//! * [`crate::components::form_sections`] — use `use_criteria_selector`

use crate::features::explore::search_state::{
    ExploreState, ResultDataState, SearchLifecycleState, UiChromeState,
};
use crate::models::SearchCriteria;
use dioxus::prelude::*;
use std::sync::Arc;

/// Wrapper around `Arc<T>` that compares by pointer identity.
///
/// This is useful for large immutable payloads (for example `Arc<[CompoundEntry]>`)
/// where deep `PartialEq` checks are unnecessarily expensive.
#[derive(Clone)]
pub struct ArcPtrEq<T: ?Sized>(pub Arc<T>);

impl<T: ?Sized> PartialEq for ArcPtrEq<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

/// Subscribe to a derived value from [`SearchLifecycleState`].
///
/// The component using this memo only re-renders when `f` returns a different
/// value, isolating it from result-data and UI-chrome mutations.
pub fn use_lifecycle_selector<T: PartialEq + Clone + 'static>(
    explore: Signal<ExploreState>,
    f: impl Fn(&SearchLifecycleState) -> T + 'static,
) -> Memo<T> {
    use_memo(move || f(&explore.read().lifecycle))
}

/// Subscribe to a derived value from [`ResultDataState`].
///
/// The component using this memo only re-renders when `f` returns a different
/// value, isolating it from lifecycle and UI-chrome mutations.
pub fn use_result_selector<T: PartialEq + Clone + 'static>(
    explore: Signal<ExploreState>,
    f: impl Fn(&ResultDataState) -> T + 'static,
) -> Memo<T> {
    use_memo(move || f(&explore.read().result))
}

/// Subscribe to an `Arc<T>` derived from [`ResultDataState`] using pointer
/// equality (`Arc::ptr_eq`) instead of deep value equality.
pub fn use_result_arc_selector<T: ?Sized + 'static>(
    explore: Signal<ExploreState>,
    f: impl Fn(&ResultDataState) -> Arc<T> + 'static,
) -> Memo<ArcPtrEq<T>> {
    use_memo(move || ArcPtrEq(f(&explore.read().result)))
}

/// Subscribe to a derived value from [`UiChromeState`].
///
/// The component using this memo only re-renders when `f` returns a different
/// value, isolating it from lifecycle and result-data mutations.
pub fn use_ui_selector<T: PartialEq + Clone + 'static>(
    explore: Signal<ExploreState>,
    f: impl Fn(&UiChromeState) -> T + 'static,
) -> Memo<T> {
    use_memo(move || f(&explore.read().ui))
}

/// Subscribe to a derived value from [`SearchCriteria`].
///
/// The component using this memo only re-renders when `f` returns a different
/// value, isolating it from unrelated criteria-field mutations.
pub fn use_criteria_selector<T: PartialEq + Clone + 'static>(
    criteria: Signal<SearchCriteria>,
    f: impl Fn(&SearchCriteria) -> T + 'static,
) -> Memo<T> {
    use_memo(move || f(&criteria.read()))
}

/// Snapshot of commonly-queried explore UI state flags.
/// Used to reduce signal reads and prevent unnecessary component re-renders
/// across results viewport, table, and toolbar sections.
#[derive(Clone, Copy, PartialEq)]
pub struct ExploreUiState {
    pub loading: bool,
    pub has_error: bool,
    pub searched_once: bool,
    pub has_entries: bool,
    pub has_query: bool,
    pub has_resolved_qid: bool,
}

impl ExploreUiState {
    pub fn from_explore(explore: Signal<ExploreState>) -> Self {
        let explore_read = explore.read();
        Self {
            loading: explore_read.lifecycle.loading,
            has_error: explore_read.lifecycle.error.is_some(),
            searched_once: explore_read.lifecycle.searched_once,
            has_entries: !explore_read.result.entries.is_empty(),
            has_query: explore_read.result.sparql_query.is_some(),
            has_resolved_qid: explore_read.result.resolved_qid.is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explore_ui_state_all_false_by_default() {
        let explore = ExploreState::default();
        let ui_state = ExploreUiState {
            loading: explore.lifecycle.loading,
            has_error: explore.lifecycle.error.is_some(),
            searched_once: explore.lifecycle.searched_once,
            has_entries: !explore.result.entries.is_empty(),
            has_query: explore.result.sparql_query.is_some(),
            has_resolved_qid: explore.result.resolved_qid.is_some(),
        };

        assert!(!ui_state.loading);
        assert!(!ui_state.has_error);
        assert!(!ui_state.searched_once);
        assert!(!ui_state.has_entries);
        assert!(!ui_state.has_query);
        assert!(!ui_state.has_resolved_qid);
    }
}
