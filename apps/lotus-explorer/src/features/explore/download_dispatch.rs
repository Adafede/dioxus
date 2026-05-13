// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Custom Dioxus hooks that encapsulate the download-related reactive effects.

use crate::download::{DownloadFormat, execute_download};
use crate::export;
use crate::features::explore::actions::ExploreAction;
use crate::features::explore::orchestrator::start_search;
use crate::features::explore::search_state::{dispatch_explore_action, set_signal_if_changed, ExploreState};
use crate::features::explore::types::ErrorKind;
use crate::i18n::{Locale, err_unsupported_format};
use crate::models::SearchCriteria;
use crate::repositories::LotusRepository;
use crate::utils::logging::{log_debug_evt, log_info_evt, log_warn_evt};
use dioxus::prelude::*;
use std::sync::Arc;

pub fn use_startup_effect<R: LotusRepository + Copy>(
    pending_download_format: Signal<Option<String>>,
    pending_execute: Signal<bool>,
    explore: Signal<ExploreState>,
    criteria: Signal<SearchCriteria>,
    locale: Signal<Locale>,
    repo: R,
) {
    use_effect(move || {
        let pending = pending_download_format.read().clone();
        if let Some(fmt) = pending.as_deref()
            && DownloadFormat::from_str(fmt).is_none()
        {
            log_warn_evt("download", "startup", "unsupported_format", Some(&format!("format={fmt}")));
            dispatch_explore_action(
                explore,
                ExploreAction::SearchFailed {
                    kind: ErrorKind::Validation,
                    message: err_unsupported_format(*locale.peek(), fmt),
                },
            );
            set_signal_if_changed(pending_download_format, None);
            return;
        }

        if (pending.is_some() || *pending_execute.read())
            && !explore.peek().searched_once
            && !explore.peek().loading
        {
            if let Some(fmt) = pending.as_deref() {
                log_info_evt("download", "startup", "auto_search_triggered", Some(&format!("format={fmt}")));
            } else {
                log_info_evt("search", "startup", "auto_search_triggered", Some("execute=true"));
            }
            start_search(criteria, locale, pending.is_some(), explore, repo);
            set_signal_if_changed(pending_execute, false);
        }
    });
}

pub fn use_download_dispatch_effect(
    pending_download_format: Signal<Option<String>>,
    explore: Signal<ExploreState>,
    locale: Signal<Locale>,
    waiting_loading_logged: Signal<bool>,
    waiting_query_logged: Signal<bool>,
) {
    use_effect(move || {
        let pending = pending_download_format.read().clone();
        let Some(fmt) = pending else {
            set_signal_if_changed(waiting_loading_logged, false);
            set_signal_if_changed(waiting_query_logged, false);
            return;
        };

        if explore.read().loading {
            if !*waiting_loading_logged.peek() {
                log_debug_evt("download", "dispatch", "waiting_loading", Some(&format!("format={fmt}")));
                set_signal_if_changed(waiting_loading_logged, true);
            }
            set_signal_if_changed(waiting_query_logged, false);
            return;
        }
        set_signal_if_changed(waiting_loading_logged, false);

        let Some(query) = explore.read().sparql_query.as_deref().map(str::to_string) else {
            if !*waiting_query_logged.peek() {
                log_debug_evt("download", "dispatch", "waiting_query", Some(&format!("format={fmt}")));
                set_signal_if_changed(waiting_query_logged, true);
            }
            return;
        };
        set_signal_if_changed(waiting_query_logged, false);

        let crit = explore.read().executed_criteria.clone();
        match DownloadFormat::from_str(&fmt) {
            Some(format) => {
                let filename = export::generate_filename(&crit, format.extension());
                set_signal_if_changed(pending_download_format, None);
                dispatch_explore_action(explore, ExploreAction::DownloadDispatchStarted);
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
                    log_info_evt("download", "dispatch", "started", Some(&format!("format={}", format.log_name())));
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
                    dispatch_explore_action(explore, ExploreAction::DownloadDispatchFinished);
                });
            }
            None => {
                log_warn_evt("download", "dispatch", "unsupported_format", Some(&format!("format={fmt}")));
                dispatch_explore_action(
                    explore,
                    ExploreAction::SearchFailed {
                        kind: ErrorKind::Validation,
                        message: err_unsupported_format(*locale.peek(), &fmt),
                    },
                );
                set_signal_if_changed(pending_download_format, None);
                dispatch_explore_action(explore, ExploreAction::DownloadDispatchFinished);
            }
        }
    });
}
