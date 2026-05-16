// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Custom Dioxus hooks that encapsulate the download-related reactive effects.
//!
//! These hooks coordinate two independent effect scenarios:
//! 1. **Startup Effect**: Decides whether to auto-trigger search based on URL parameters.
//! 2. **Dispatch Effect**: Monitors search progress and coordinates the download phase once results ready.
//!
//! See [`super::download_effects`] for pure business logic separated from Dioxus hooks.

use crate::app_state::AppState;
use crate::download::execute_download;
use crate::features::explore::actions::ExploreAction;
use crate::features::explore::command::SearchCommand;
use crate::features::explore::download_effects::{self, DispatchPhase, StartupTriggerMode};
use crate::features::explore::orchestrator::{SearchTaskController, start_search};
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
    search_tasks: SearchTaskController,
    repo: R,
) {
    let repo_for_effect = repo.clone();
    use_effect(move || {
        let repo = repo_for_effect.clone();
        let pending = app_state.read().download.pending_format;
        let invalid_pending = app_state.read().download.pending_invalid_format.clone();
        if let Some(fmt) = invalid_pending {
            telemetry::download_startup_unsupported_format(&fmt);
            dispatch_explore_action(
                explore,
                ExploreAction::SearchFailed {
                    error: DomainError::Validation(ValidationFault::UnsupportedFormat {
                        format: fmt,
                    }),
                },
            );
            app_state.with_mut(|state| {
                if state.download.pending_invalid_format.is_some() {
                    state.download.pending_invalid_format = None;
                }
            });
            return;
        }

        let explore_read = explore.peek();
        if download_effects::should_trigger_startup_search(
            pending,
            app_state.read().download.direct_execute,
            explore_read.lifecycle.searched_once,
            explore_read.lifecycle.loading,
        ) {
            let (trigger_mode, command) = if let Some(format) = pending {
                (
                    StartupTriggerMode::Download { format },
                    SearchCommand::StartupDownload,
                )
            } else {
                (
                    StartupTriggerMode::DirectExecute,
                    SearchCommand::StartupExecute,
                )
            };
            trigger_mode.log();

            start_search(criteria, command, explore, search_tasks.clone(), repo);
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
        let pending = app_state.read().download.pending_format;
        let explore_state = explore.read();

        // Classify current phase based on pending format and explore state.
        let phase = download_effects::classify_dispatch_phase(pending, &explore_state);

        match phase {
            DispatchPhase::Inactive => {
                // No download pending — reset any logging guards.
                let metrics = app_state.peek().metrics.clone();
                if metrics.waiting_loading_logged || metrics.waiting_query_logged {
                    app_state.with_mut(|state| {
                        state.metrics.waiting_loading_logged = false;
                        state.metrics.waiting_query_logged = false;
                    });
                }
            }
            DispatchPhase::WaitingForLoading => {
                // Still loading — log once per cycle via guard.
                if !app_state.peek().metrics.waiting_loading_logged {
                    telemetry::download_dispatch_waiting_loading(pending.unwrap().log_name());
                    app_state.with_mut(|state| state.metrics.waiting_loading_logged = true);
                }
                // Reset query-waiting guard since we're focused on loading now.
                if app_state.peek().metrics.waiting_query_logged {
                    app_state.with_mut(|state| state.metrics.waiting_query_logged = false);
                }
            }
            DispatchPhase::WaitingForQuery => {
                // Loading complete, waiting for query — log once via guard.
                if !app_state.peek().metrics.waiting_query_logged {
                    telemetry::download_dispatch_waiting_query(pending.unwrap().log_name());
                    app_state.with_mut(|state| state.metrics.waiting_query_logged = true);
                }
                // Reset loading guard since loading has finished.
                if app_state.peek().metrics.waiting_loading_logged {
                    app_state.with_mut(|state| state.metrics.waiting_loading_logged = false);
                }
            }
            DispatchPhase::Ready {
                query,
                filename,
                format,
                #[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
                criteria,
            } => {
                // All preconditions met — clear pending download and start dispatch.
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
                        Arc::new(criteria.clone()),
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
        }
    });
}
