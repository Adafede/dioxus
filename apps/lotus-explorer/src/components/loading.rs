// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Loading and download-dispatching overlay components.
//!
//! These components are intentionally small so that phase-text transitions
//! (e.g., ResolvingTaxon → Counting → FetchingPreview) only re-render
//! the component that subscribes to `query_phase`, not the entire
//! `ResultsViewport` tree.

use crate::features::explore::types::QueryPhase;
use crate::i18n::{Locale, TextKey, t};
use crate::state::use_results_context;
use dioxus::prelude::*;

/// Spinner overlay shown while a query is in-flight.
///
/// Subscribes to `query_phase` independently so phase-text updates do not
/// propagate to `ResultsViewport` or its siblings.
#[component]
pub fn LoadingState(locale: Locale) -> Element {
    let state = use_results_context();
    let query_phase = *state.query_phase.read();
    rsx! {
        div {
            class: "loading-state",
            role: "status",
            aria_live: "polite",
            aria_busy: "true",
            div { class: "spinner-lg", "aria-hidden": "true" }
            p { "{query_phase_text(locale, query_phase)}" }
            p { class: "loading-hint", "{t(locale, TextKey::LoadingHint)}" }
        }
    }
}

/// Spinner shown while a download file is being assembled.
#[component]
pub fn DownloadDispatchState(locale: Locale) -> Element {
    rsx! {
        div {
            class: "loading-state",
            role: "status",
            aria_live: "polite",
            aria_busy: "true",
            div { class: "spinner-lg", "aria-hidden": "true" }
            p { "{t(locale, TextKey::PreparingDownload)}" }
            p { class: "loading-hint", "{t(locale, TextKey::WelcomeProgrammaticDownload)}" }
        }
    }
}

/// Notice shown when the URL triggered a download-only mode but the SPARQL
/// query has not materialised yet, offering the user a "Run search" escape.
#[component]
pub fn DownloadOnlyState(locale: Locale, on_preview: EventHandler<()>) -> Element {
    rsx! {
        div { class: "notice notice-info", role: "status",
            span { class: "notice-label", "{t(locale, TextKey::Notice)}" }
            span { class: "notice-value", "{t(locale, TextKey::WelcomeProgrammaticDownload)}" }
            button {
                class: "btn btn-sm",
                r#type: "button",
                onclick: move |_| on_preview.call(()),
                "{t(locale, TextKey::RunSearch)}"
            }
        }
    }
}

// ── Pure helpers ──────────────────────────────────────────────────────────────

/// Maps a `QueryPhase` to the user-facing loading-state label.
pub fn query_phase_text(locale: Locale, phase: QueryPhase) -> &'static str {
    match phase {
        QueryPhase::Idle => t(locale, TextKey::LoadingTitle),
        QueryPhase::ResolvingTaxon => t(locale, TextKey::LoadingResolvingTaxon),
        QueryPhase::Counting => t(locale, TextKey::LoadingCounting),
        QueryPhase::FetchingPreview => t(locale, TextKey::LoadingFetchingPreview),
        QueryPhase::Rendering => t(locale, TextKey::LoadingRendering),
    }
}
