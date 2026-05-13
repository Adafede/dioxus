// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::explore::search_state::ExploreState;
use crate::models::*;
use dioxus::prelude::*;

/// Context for the search form and controls.
#[derive(Clone, Copy)]
pub struct SearchUiContext {
    pub criteria: Signal<SearchCriteria>,
    pub explore: Signal<ExploreState>,
}

impl SearchUiContext {
    pub fn from_signals(criteria: Signal<SearchCriteria>, explore: Signal<ExploreState>) -> Self {
        Self { criteria, explore }
    }
}

/// Context for result rendering and result-driven actions.
#[derive(Clone, Copy)]
pub struct ResultsContext {
    pub explore: Signal<ExploreState>,
}

impl ResultsContext {
    pub fn from_signals(explore: Signal<ExploreState>) -> Self {
        Self { explore }
    }
}

pub fn use_search_ui_context() -> SearchUiContext {
    use_context::<SearchUiContext>()
}

pub fn use_results_context() -> ResultsContext {
    use_context::<ResultsContext>()
}
