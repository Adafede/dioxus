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
