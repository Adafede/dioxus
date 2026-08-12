// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Loading and download-dispatching overlay components.
//!
//! These components are intentionally small so that phase-text transitions
//! (e.g., ResolvingTaxon -> FetchingResults -> ProcessingResults) only re-render
//! the component that subscribes to `query_phase`, not the entire
//! `ResultsViewport` tree.

use crate::features::explore::interactions::use_explore_interactions;
use crate::features::explore::selectors::use_lifecycle_selector;
use crate::features::explore::types::QueryPhase;
use crate::i18n::{Locale, TextKey, t};
use crate::state::use_results_context;
use dioxus::prelude::*;
use ui::prelude::*;

/// Spinner overlay shown while a query is in-flight.
///
/// Subscribes to `query_phase` independently so phase-text updates do not
/// propagate to `ResultsViewport` or its siblings.
#[component]
pub fn LoadingState() -> Element {
    let locale = crate::hooks::use_locale();
    let explore = use_results_context().explore;
    let query_phase = *use_lifecycle_selector(explore, |lifecycle| lifecycle.query_phase).read();
    rsx! {
        div {
            role: "status",
            aria_live: "polite",
            aria_busy: "true",
            style: loading_state_style(),
            div { style: spinner_lg_style(), "aria-hidden": "true" }
            p { "{query_phase_text(locale, query_phase)}" }
            p { style: hint_style(), "{t(locale, TextKey::LoadingHint)}" }
        }
    }
}

/// Spinner shown while a download file is being assembled.
#[component]
pub fn DownloadDispatchState() -> Element {
    let locale = crate::hooks::use_locale();
    rsx! {
        div {
            role: "status",
            aria_live: "polite",
            aria_busy: "true",
            style: loading_state_style(),
            div { style: spinner_lg_style(), "aria-hidden": "true" }
            p { "{t(locale, TextKey::PreparingDownload)}" }
            p { style: hint_style(), "{t(locale, TextKey::WelcomeProgrammaticDownload)}" }
        }
    }
}

/// Notice shown when the URL triggered a download-only mode but the SPARQL
/// query has not materialized yet, offering the user a "Run search" escape.
#[component]
pub fn DownloadOnlyState() -> Element {
    let locale = crate::hooks::use_locale();
    let interactions = use_explore_interactions();
    rsx! {
        NoticeBar {
            label: t(locale, TextKey::Notice).to_string(),
            tone: NoticeTone::Warning,
            role: "status",
            aria_live: "polite",
            span { style: notice_value_style(), "{t(locale, TextKey::WelcomeProgrammaticDownload)}" }
            button {
                r#type: "button",
                style: button_base_style(),
                onclick: move |_| interactions.preview(),
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
        QueryPhase::PreparingQuery => t(locale, TextKey::LoadingTitle),
        QueryPhase::ResolvingTaxon => t(locale, TextKey::LoadingResolvingTaxon),
        QueryPhase::FetchingResults => t(locale, TextKey::LoadingFetchingResults),
        QueryPhase::ProcessingResults => t(locale, TextKey::LoadingProcessingResults),
        QueryPhase::Rendering => t(locale, TextKey::LoadingRendering),
    }
}

fn spinner_lg_style() -> String {
    StyleBuilder::new()
        .property("width", "40px")
        .property("height", "40px")
        .border("3px solid var(--border)")
        .property("border-top-color", "var(--accent)")
        .border_radius("50%")
        .property("animation", "spin .8s linear infinite")
        .build()
}

fn hint_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-0)")
        .color("var(--text3)")
        .build()
}

fn loading_state_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .align_items("center")
        .justify_content("center")
        .gap("14px")
        .padding("48px")
        .color("var(--text2)")
        .property("flex", "1")
        .build()
}

fn notice_value_style() -> String {
    crate::ui::style_constants::shared::notice_value_style()
}

fn button_base_style() -> String {
    crate::ui::style_constants::buttons::button_base_style()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preparing_phase_uses_generic_loading_title() {
        assert_eq!(
            query_phase_text(Locale::En, QueryPhase::PreparingQuery),
            query_phase_text(Locale::En, QueryPhase::Idle)
        );
    }
}
