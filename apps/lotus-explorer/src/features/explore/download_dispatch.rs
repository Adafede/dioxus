// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Custom Dioxus hooks that encapsulate the download-related reactive effects.

use crate::app_state::AppState;
use crate::download::{DownloadFormat, execute_download};
use crate::export;
use crate::features::explore::actions::ExploreAction;
use crate::features::explore::orchestrator::start_search;
use crate::features::explore::search_state::{ExploreState, dispatch_explore_action};
use crate::features::explore::types::{DomainError, ValidationFault};
use crate::models::SearchCriteria;
use crate::repositories::LotusRepository;
use crate::services::search_telemetry as telemetry;
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;

pub fn use_startup_effect<R: LotusRepository>(
    mut app_state: Signal<AppState>,
    explore: Signal<ExploreState>,
    criteria: Signal<SearchCriteria>,
    repo: R,
) {
    let repo_for_effect = repo.clone();
    use_effect(move || {
        let repo = repo_for_effect.clone();
        let pending = app_state.read().download.pending_format.clone();
        if let Some(fmt) = pending.as_deref()
            && DownloadFormat::from_str(fmt).is_none()
        {
            telemetry::download_startup_unsupported_format(fmt);
            dispatch_explore_action(
                explore,
                ExploreAction::SearchFailed {
                    error: DomainError::Validation(ValidationFault::UnsupportedFormat {
                        format: fmt.to_string(),
                    }),
                },
            );
            app_state.with_mut(|state| {
                if state.download.pending_format.is_some() {
                    state.download.pending_format = None;
                }
            });
            return;
        }

        if (pending.is_some() || app_state.read().download.direct_execute)
            && !explore.peek().lifecycle.searched_once
            && !explore.peek().lifecycle.loading
        {
            if let Some(fmt) = pending.as_deref() {
                telemetry::download_startup_auto_search_triggered(fmt);
            } else {
                telemetry::search_startup_auto_search_execute();
            }
            start_search(criteria, pending.is_some(), explore, repo);
            app_state.with_mut(|state| {
                if state.download.direct_execute {
                    state.download.direct_execute = false;
                }
            });
        }
    });
}

pub fn use_download_dispatch_effect(
    mut app_state: Signal<AppState>,
    explore: Signal<ExploreState>,
) {
    use_effect(move || {
        let pending = app_state.read().download.pending_format.clone();
        let Some(fmt) = pending else {
            let metrics = app_state.peek().metrics.clone();
            if metrics.waiting_loading_logged || metrics.waiting_query_logged {
                app_state.with_mut(|state| {
                    state.metrics.waiting_loading_logged = false;
                    state.metrics.waiting_query_logged = false;
                });
            }
            return;
        };

        if explore.read().lifecycle.loading {
            if !app_state.peek().metrics.waiting_loading_logged {
                telemetry::download_dispatch_waiting_loading(&fmt);
                app_state.with_mut(|state| state.metrics.waiting_loading_logged = true);
            }
            if app_state.peek().metrics.waiting_query_logged {
                app_state.with_mut(|state| state.metrics.waiting_query_logged = false);
            }
            return;
        }
        if app_state.peek().metrics.waiting_loading_logged {
            app_state.with_mut(|state| state.metrics.waiting_loading_logged = false);
        }

        let Some(query) = explore
            .read()
            .result
            .sparql_query
            .as_deref()
            .map(str::to_string)
        else {
            if !app_state.peek().metrics.waiting_query_logged {
                telemetry::download_dispatch_waiting_query(&fmt);
                app_state.with_mut(|state| state.metrics.waiting_query_logged = true);
            }
            return;
        };
        if app_state.peek().metrics.waiting_query_logged {
            app_state.with_mut(|state| state.metrics.waiting_query_logged = false);
        }

        let crit = explore.read().ui.executed_criteria.clone();
        match DownloadFormat::from_str(&fmt) {
            Some(format) => {
                let filename = export::generate_filename(&crit, format.extension());
                if app_state.peek().download.pending_format.is_some() {
                    app_state.with_mut(|state| {
                        state.download.pending_format = None;
                    });
                }
                dispatch_explore_action(explore, ExploreAction::DownloadDispatchStarted);
                telemetry::download_startup_dispatch_query_check(
                    format.log_name(),
                    query.contains("SERVICE"),
                    query.contains("SELECT"),
                    query.len(),
                );
                spawn(async move {
                    telemetry::download_dispatch_started(format.log_name());
                    if let Err(err) = execute_download(
                        format,
                        #[cfg(target_arch = "wasm32")]
                        Arc::new(crit.clone()),
                        query,
                        filename,
                    )
                    .await
                    {
                        telemetry::download_dispatch_error(format.log_name(), &err.to_string());
                    }
                    dispatch_explore_action(explore, ExploreAction::DownloadDispatchFinished);
                });
            }
            None => {
                telemetry::download_dispatch_unsupported_format(&fmt);
                dispatch_explore_action(
                    explore,
                    ExploreAction::SearchFailed {
                        error: DomainError::Validation(ValidationFault::UnsupportedFormat {
                            format: fmt.clone(),
                        }),
                    },
                );
                if app_state.peek().download.pending_format.is_some() {
                    app_state.with_mut(|state| {
                        state.download.pending_format = None;
                    });
                }
                dispatch_explore_action(explore, ExploreAction::DownloadDispatchFinished);
            }
        }
    });
}
