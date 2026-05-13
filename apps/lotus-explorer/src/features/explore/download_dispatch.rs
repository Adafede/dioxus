// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Custom Dioxus hooks that encapsulate the two download-related reactive
//! effects previously inlined into the root `App()` component.
//!
//! # Rules
//! Both `use_startup_effect` and `use_download_dispatch_effect` call
//! `use_effect` internally. They must be called **unconditionally** at the
//! component level, exactly like any other Dioxus hook.

use crate::download::{DownloadFormat, execute_download};
use crate::export;
use crate::features::explore::orchestrator::start_search;
use crate::features::explore::search_state::{SearchRuntime, set_signal_if_changed};
use crate::features::explore::types::ErrorKind;
use crate::i18n::{Locale, err_unsupported_format};
use crate::models::SearchCriteria;
use crate::repositories::LotusRepository;
use crate::utils::logging::{log_debug_evt, log_info_evt, log_warn_evt};
use dioxus::prelude::*;
use std::sync::Arc;

/// Registers a reactive effect that triggers an automatic search (or download)
/// on startup when the URL contains `execute=true` or `download=true&format=…`.
///
/// Must be called exactly once, unconditionally, inside a component.
pub fn use_startup_effect<R: LotusRepository + Copy>(
    pending_download_format: Signal<Option<String>>,
    pending_execute: Signal<bool>,
    searched_once: Signal<bool>,
    loading: Signal<bool>,
    locale: Signal<Locale>,
    error: Signal<Option<String>>,
    error_kind: Signal<ErrorKind>,
    criteria: Signal<SearchCriteria>,
    search_runtime: SearchRuntime,
    repo: R,
) {
    use_effect(move || {
        let pending = pending_download_format.read().clone();

        // Validate format string eagerly so the user gets a clear validation
        // error immediately rather than a silent missing download.
        if let Some(fmt) = pending.as_deref()
            && DownloadFormat::from_str(fmt).is_none()
        {
            log_warn_evt(
                "download",
                "startup",
                "unsupported_format",
                Some(&format!("format={fmt}")),
            );
            set_signal_if_changed(error_kind, ErrorKind::Validation);
            set_signal_if_changed(error, Some(err_unsupported_format(*locale.peek(), fmt)));
            set_signal_if_changed(pending_download_format, None);
            return;
        }

        if (pending.is_some() || *pending_execute.read())
            && !*searched_once.read()
            && !*loading.read()
        {
            if let Some(fmt) = pending.as_deref() {
                log_info_evt(
                    "download",
                    "startup",
                    "auto_search_triggered",
                    Some(&format!("format={fmt}")),
                );
            } else {
                log_info_evt(
                    "search",
                    "startup",
                    "auto_search_triggered",
                    Some("execute=true"),
                );
            }
            start_search(criteria, locale, pending.is_some(), search_runtime, repo);
            set_signal_if_changed(pending_execute, false);
        }
    });
}

/// Registers a reactive effect that watches for a materialised SPARQL query
/// and kicks off the file download once it arrives.
///
/// The effect is split from the startup trigger to avoid redundant re-runs:
/// the startup effect fires once, whereas the dispatch effect polls until the
/// query is ready.
///
/// `waiting_loading_logged` and `waiting_query_logged` are internal guard
/// signals that suppress duplicate debug log lines.  They should never be
/// subscribed to in RSX; access them only via `.peek()`.
///
/// Must be called exactly once, unconditionally, inside a component.
pub fn use_download_dispatch_effect(
    pending_download_format: Signal<Option<String>>,
    loading: Signal<bool>,
    sparql_query: Signal<Option<Arc<str>>>,
    criteria: Signal<SearchCriteria>,
    locale: Signal<Locale>,
    error: Signal<Option<String>>,
    error_kind: Signal<ErrorKind>,
    download_dispatching: Signal<bool>,
    waiting_loading_logged: Signal<bool>,
    waiting_query_logged: Signal<bool>,
) {
    use_effect(move || {
        let pending = pending_download_format.read().clone();

        let Some(fmt) = pending else {
            // No pending download — reset guard flags.
            set_signal_if_changed(waiting_loading_logged, false);
            set_signal_if_changed(waiting_query_logged, false);
            return;
        };

        // Search still in-flight — wait.
        if *loading.read() {
            if !*waiting_loading_logged.peek() {
                log_debug_evt(
                    "download",
                    "dispatch",
                    "waiting_loading",
                    Some(&format!("format={fmt}")),
                );
                set_signal_if_changed(waiting_loading_logged, true);
            }
            set_signal_if_changed(waiting_query_logged, false);
            return;
        }
        set_signal_if_changed(waiting_loading_logged, false);

        // SPARQL query not yet produced — wait.
        let Some(query) = sparql_query.read().as_deref().map(str::to_string) else {
            if !*waiting_query_logged.peek() {
                log_debug_evt(
                    "download",
                    "dispatch",
                    "waiting_query",
                    Some(&format!("format={fmt}")),
                );
                set_signal_if_changed(waiting_query_logged, true);
            }
            return;
        };
        set_signal_if_changed(waiting_query_logged, false);

        let crit = criteria.peek().clone();
        match DownloadFormat::from_str(&fmt) {
            Some(format) => {
                let filename = export::generate_filename(&crit, format.extension());
                set_signal_if_changed(pending_download_format, None);
                set_signal_if_changed(download_dispatching, true);
                log_debug_evt(
                    "download",
                    "startup_dispatch",
                    "query_check",
                    Some(&format!(
                        "format={} has_SERVICE={} has_SELECT={} query_bytes={}",
                        format.log_name(),
                        query.contains("SERVICE"),
                        query.contains("SELECT"),
                        query.len()
                    )),
                );
                spawn(async move {
                    log_info_evt(
                        "download",
                        "dispatch",
                        "started",
                        Some(&format!("format={}", format.log_name())),
                    );
                    if let Err(err) = execute_download(
                        format,
                        #[cfg(target_arch = "wasm32")]
                        Arc::new(crit.clone()),
                        query,
                        filename,
                    )
                    .await
                    {
                        log_warn_evt(
                            "download",
                            "dispatch",
                            "error",
                            Some(&format!("format={} reason={err}", format.log_name())),
                        );
                    }
                    set_signal_if_changed(download_dispatching, false);
                });
            }
            None => {
                log_warn_evt(
                    "download",
                    "dispatch",
                    "unsupported_format",
                    Some(&format!("format={fmt}")),
                );
                set_signal_if_changed(error_kind, ErrorKind::Validation);
                set_signal_if_changed(error, Some(err_unsupported_format(*locale.peek(), &fmt)));
                set_signal_if_changed(pending_download_format, None);
                set_signal_if_changed(download_dispatching, false);
            }
        }
    });
}

