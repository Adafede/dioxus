// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Top-level results area component.
//!
//! `ResultsViewport` decides which child to render based on the coarse
//! loading/error/empty state, deliberately subscribing to as few signals as
//! possible so that per-phase loading-text updates don't cascade here.

use crate::components::loading::{DownloadDispatchState, DownloadOnlyState, LoadingState};
use crate::components::results_table::ResultsTable;
use crate::components::welcome::WelcomeScreen;
use crate::state::use_results_context;
use dioxus::prelude::*;

/// Selects which result view to show.
///
/// Subscribes narrowly via fine-grained selectors — phase-specific text lives
/// inside `LoadingState` so phase transitions do not cascade here.
#[component]
pub fn ResultsViewport(on_preview: EventHandler<()>) -> Element {
    use crate::features::explore::selectors::{use_lifecycle_selector, use_result_selector};

    let state = use_results_context();
    let explore = state.explore;

    // Fine-grained selectors: only re-render this component when these specific
    // fields change, not on every ExploreState mutation.
    let loading = use_lifecycle_selector(explore, |lc| lc.loading);
    let has_error = use_lifecycle_selector(explore, |lc| lc.error.is_some());
    let searched_once = use_lifecycle_selector(explore, |lc| lc.searched_once);
    let download_only_mode = use_lifecycle_selector(explore, |lc| lc.download_only_mode);
    let download_dispatching = use_lifecycle_selector(explore, |lc| lc.download_dispatching);
    let entries_empty = use_result_selector(explore, |r| r.entries.is_empty());

    let loading = *loading.read();
    let has_error = *has_error.read();
    let searched_once = *searched_once.read();
    let download_only_mode = *download_only_mode.read();
    let download_dispatching = *download_dispatching.read();
    let entries_is_empty = *entries_empty.read();

    if loading {
        return rsx! {
            LoadingState {}
        };
    }

    if entries_is_empty && !has_error && !searched_once {
        return rsx! {
            WelcomeScreen {}
        };
    }

    if entries_is_empty && !has_error && download_only_mode && download_dispatching {
        return rsx! {
            DownloadDispatchState {}
        };
    }

    if entries_is_empty && !has_error && download_only_mode {
        return rsx! {
            DownloadOnlyState { on_preview }
        };
    }

    rsx! {
        ResultsTable {}
    }
}
