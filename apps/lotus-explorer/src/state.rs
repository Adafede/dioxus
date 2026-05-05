// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::QueryPhase;
use crate::i18n::Locale;
use crate::models::*;
use dioxus::prelude::*;
use std::sync::Arc;

/// Context for the search form and controls.
#[derive(Clone, Copy)]
pub struct SearchUiContext {
    pub criteria: Signal<SearchCriteria>,
    pub locale: Signal<Locale>,
    pub loading: Signal<bool>,
}

impl SearchUiContext {
    pub fn from_signals(
        criteria: Signal<SearchCriteria>,
        locale: Signal<Locale>,
        loading: Signal<bool>,
    ) -> Self {
        Self {
            criteria,
            locale,
            loading,
        }
    }
}

/// Context for result rendering and result-driven actions.
#[derive(Clone, Copy)]
pub struct ResultsContext {
    pub executed_criteria: Signal<SearchCriteria>,
    pub locale: Signal<Locale>,
    pub entries: Signal<Rows>,
    pub loading: Signal<bool>,
    pub error: Signal<Option<String>>,
    pub query_phase: Signal<QueryPhase>,
    pub searched_once: Signal<bool>,
    pub download_only_mode: Signal<bool>,
    pub download_dispatching: Signal<bool>,
    pub query_hash: Signal<Option<String>>,
    pub result_hash: Signal<Option<String>>,
    pub sparql_query: Signal<Option<Arc<str>>>,
    pub metadata_json: Signal<Option<Arc<str>>>,
    pub total_matches: Signal<Option<usize>>,
    pub total_stats: Signal<Option<DatasetStats>>,
    pub display_capped_rows: Signal<bool>,
    pub sort: Signal<SortState>,
}

impl ResultsContext {
    #[allow(clippy::too_many_arguments)]
    pub fn from_signals(
        executed_criteria: Signal<SearchCriteria>,
        locale: Signal<Locale>,
        entries: Signal<Rows>,
        loading: Signal<bool>,
        error: Signal<Option<String>>,
        query_phase: Signal<QueryPhase>,
        searched_once: Signal<bool>,
        download_only_mode: Signal<bool>,
        download_dispatching: Signal<bool>,
        query_hash: Signal<Option<String>>,
        result_hash: Signal<Option<String>>,
        sparql_query: Signal<Option<Arc<str>>>,
        metadata_json: Signal<Option<Arc<str>>>,
        total_matches: Signal<Option<usize>>,
        total_stats: Signal<Option<DatasetStats>>,
        display_capped_rows: Signal<bool>,
        sort: Signal<SortState>,
    ) -> Self {
        Self {
            executed_criteria,
            locale,
            entries,
            loading,
            error,
            query_phase,
            searched_once,
            download_only_mode,
            download_dispatching,
            query_hash,
            result_hash,
            sparql_query,
            metadata_json,
            total_matches,
            total_stats,
            display_capped_rows,
            sort,
        }
    }
}

pub fn use_search_ui_context() -> SearchUiContext {
    use_context::<SearchUiContext>()
}

pub fn use_results_context() -> ResultsContext {
    use_context::<ResultsContext>()
}
