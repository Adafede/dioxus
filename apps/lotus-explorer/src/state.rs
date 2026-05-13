// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::explore::form_actions::{FormAction, apply_form_action};
use crate::features::explore::search_state::ExploreState;
use crate::models::*;
use dioxus::prelude::*;

// ── Form Criteria Context ─────────────────────────────────────────────────

/// Context provider for search form with unified action dispatch.
///
/// Replaces 18+ individual event handlers with a single `update()` method.
/// This enables granular components to avoid props drilling entirely.
///
/// ## Usage
///
/// ```rust,ignore
/// let ctx = use_form_criteria_context();
/// ctx.update(FormAction::Taxon("Quercus".to_string()));
/// ```
#[derive(Clone, Copy)]
pub struct FormCriteriaContext {
    #[allow(dead_code)] // Will be accessed when SearchPanel is refactored
    pub criteria: Signal<SearchCriteria>,
}

impl FormCriteriaContext {
    pub fn new(criteria: Signal<SearchCriteria>) -> Self {
        Self { criteria }
    }

    /// Dispatch a form action, mutating the criteria signal.
    #[allow(dead_code)] // Will be used when SearchPanel is refactored to use context
    pub fn update(&self, action: FormAction) {
        let mut criteria = self.criteria;
        let current = criteria.peek().clone();
        let updated = apply_form_action(current, action);
        *criteria.write() = updated;
    }

    #[allow(dead_code)] // Will be used when SearchPanel is refactored to use context
    pub fn peek(&self) -> SearchCriteria {
        self.criteria.peek().clone()
    }
}

// ── Search UI Context (legacy, for compatibility) ─────────────────────────

/// Context for the search form and controls. (Will be refactored in later phase)
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

// ── Results Context ───────────────────────────────────────────────────────

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

// ── Hook Helpers ──────────────────────────────────────────────────────────

pub fn use_search_ui_context() -> SearchUiContext {
    use_context::<SearchUiContext>()
}

pub fn use_results_context() -> ResultsContext {
    use_context::<ResultsContext>()
}

#[allow(dead_code)] // Will be used when SearchPanel is refactored to use FormCriteriaContext
pub fn use_form_criteria_context() -> FormCriteriaContext {
    use_context::<FormCriteriaContext>()
}
