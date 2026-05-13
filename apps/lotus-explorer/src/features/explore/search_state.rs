// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::explore::types::{ErrorKind, QueryPhase};
use crate::models::{DatasetStats, Rows, SearchCriteria};
use dioxus::prelude::*;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct SearchRuntime {
    pub executed_criteria: Signal<SearchCriteria>,
    pub loading: Signal<bool>,
    pub error: Signal<Option<String>>,
    pub error_kind: Signal<ErrorKind>,
    pub query_phase: Signal<QueryPhase>,
    pub searched_once: Signal<bool>,
    pub download_only_mode: Signal<bool>,
    pub download_dispatching: Signal<bool>,
    pub entries: Signal<Rows>,
    pub taxon_notice: Signal<Option<String>>,
    pub resolved_qid: Signal<Option<String>>,
    pub query_hash: Signal<Option<String>>,
    pub result_hash: Signal<Option<String>>,
    pub sparql_query: Signal<Option<Arc<str>>>,
    pub metadata_json: Signal<Option<Arc<str>>>,
    pub total_matches: Signal<Option<usize>>,
    pub total_stats: Signal<Option<DatasetStats>>,
    pub display_capped_rows: Signal<bool>,
    pub mobile_filters_open: Signal<bool>,
    pub search_request_token: Signal<u64>,
}

#[derive(Default, Clone, Copy)]
pub struct SearchMetrics {
    pub network_ms: f64,
    pub parse_ms: f64,
    pub sparql_calls: usize,
}

impl SearchMetrics {
    pub fn add_network(&mut self, elapsed: std::time::Duration) {
        self.network_ms += elapsed.as_secs_f64() * 1000.0;
        self.sparql_calls += 1;
    }

    pub fn add_parse(&mut self, elapsed: std::time::Duration) {
        self.parse_ms += elapsed.as_secs_f64() * 1000.0;
    }
}

pub fn emit_search_summary(total_elapsed: std::time::Duration, metrics: SearchMetrics) {
    let total_ms = total_elapsed.as_secs_f64() * 1000.0;
    let details = format!(
        "total_ms={total_ms:.1} network_ms={:.1} parse_ms={:.1} sparql_calls={}",
        metrics.network_ms, metrics.parse_ms, metrics.sparql_calls
    );
    crate::utils::logging::log_info_evt("search", "summary", "done", Some(&details));

    if total_ms >= 5000.0 {
        crate::utils::logging::log_warn_evt("search", "summary", "slow_query", Some(&details));
    }
}

pub fn set_signal_if_changed<T: PartialEq + 'static>(mut signal: Signal<T>, next: T) {
    if *signal.peek() != next {
        *signal.write() = next;
    }
}
