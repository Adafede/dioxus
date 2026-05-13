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
/// Subscribes to: `loading`, `entries` (is_empty), `error` (is_some),
/// `searched_once`, `download_only_mode`, `download_dispatching`.
/// Phase-specific text lives inside `LoadingState` so only that component
/// re-renders on phase transitions.
#[component]
pub fn ResultsViewport(on_preview: EventHandler<()>) -> Element {
    let state = use_results_context();
    let locale = *state.locale.read();
    let loading = *state.loading.read();
    let has_error = state.error.read().is_some();
    let searched_once = *state.searched_once.read();
    let download_only_mode = *state.download_only_mode.read();
    let download_dispatching = *state.download_dispatching.read();
    let entries = state.entries;

    if loading {
        return rsx! {
            LoadingState { locale }
        };
    }

    if entries.read().is_empty() && !has_error && !searched_once {
        return rsx! {
            WelcomeScreen { locale }
        };
    }

    if entries.read().is_empty() && !has_error && download_only_mode && download_dispatching {
        return rsx! {
            DownloadDispatchState { locale }
        };
    }

    if entries.read().is_empty() && !has_error && download_only_mode {
        return rsx! {
            DownloadOnlyState { locale, on_preview }
        };
    }

    rsx! {
        ResultsTable {}
    }
}
