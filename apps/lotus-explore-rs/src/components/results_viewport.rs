// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Top-level results area component using phase-driven rendering.

use crate::components::loading::{DownloadDispatchState, DownloadOnlyState, LoadingState};
use crate::components::results_table::ResultsTable;
use crate::components::welcome::WelcomeScreen;
use crate::state::use_results_context;
use crate::ui::ContentPhase;
use dioxus::prelude::*;
use ui::prelude::*;

#[component]
pub fn ResultsViewport() -> Element {
    use crate::features::explore::ExploreUiState;
    use crate::features::explore::selectors::use_result_selector;

    let state = use_results_context();
    let explore = state.explore;
    // Hoisted to component top-level — hooks must be called unconditionally.
    let _locale = crate::hooks::use_locale();

    let ui_state = use_memo(move || ExploreUiState::from_explore(explore));

    let phase = use_memo(move || {
        let s = *ui_state.read();
        ContentPhase::from_lifecycle(
            s.loading,
            s.has_error,
            s.searched_once,
            s.download_only_mode,
            s.has_entries,
        )
    });

    // Get the SPARQL query to show even on error
    let sparql_query = use_result_selector(explore, |result| result.sparql_query.clone());

    match *phase.read() {
        ContentPhase::Welcome => rsx! {
            WelcomeScreen {}
        },
        ContentPhase::Loading => rsx! {
            LoadingState {}
        },
        // Error state: show the SPARQL query that was attempted
        ContentPhase::Error => {
            let query = sparql_query.read();
            query.as_ref().map_or_else(
                || rsx! {},
                |q| {
                    rsx! {
                        QueryDisplay { query: (*q).to_string() }
                    }
                },
            )
        }
        ContentPhase::Empty => rsx! {
            ResultsTable {}
        },
        ContentPhase::Loaded => rsx! {
            ResultsTable {}
        },
        ContentPhase::DownloadOnly => {
            if ui_state.read().download_dispatching {
                rsx! {
                    DownloadDispatchState {}
                }
            } else {
                rsx! {
                    DownloadOnlyState {}
                }
            }
        }
    }
}

#[component]
fn QueryDisplay(query: String) -> Element {
    rsx! {
        section {
            id: "query-display",
            style: query_section_style(),
            h2 { style: query_title_style(), "SPARQL Query" }
            div { style: query_container_style(),
                pre {
                    code { style: query_code_style(), "{query}" }
                }
            }
        }
    }
}

fn query_section_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("8px")
        .padding("12px 22px")
        .build()
}

fn query_title_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-1)")
        .font_weight("700")
        .color("var(--text)")
        .build()
}

fn query_container_style() -> String {
    StyleBuilder::new()
        .border("1px solid var(--panel-border)")
        .border_radius("var(--radius)")
        .background_color("var(--panel-bg-soft)")
        .box_shadow("var(--panel-shadow)")
        .build()
}

fn query_code_style() -> String {
    StyleBuilder::new()
        .display("block")
        .padding("12px 16px")
        .property("margin", "0")
        .font_family("var(--mono)")
        .font_size("var(--fs-0)")
        .color("var(--text)")
        .background_color("var(--bg2)")
        .property("border-left", "3px solid var(--wd-entries)")
        .property("white-space", "pre-wrap")
        .property("word-break", "break-word")
        .property("max-height", "320px")
        .property("overflow", "auto")
        .build()
}

#[cfg(test)]
mod tests {
    use crate::ui::ContentPhase;

    #[test]
    fn phase_welcome_when_initial_state() {
        let phase = ContentPhase::from_lifecycle(false, false, false, false, false);
        assert_eq!(phase, ContentPhase::Welcome);
    }

    #[test]
    fn phase_loading_takes_priority() {
        let phase = ContentPhase::from_lifecycle(true, false, true, false, true);
        assert_eq!(phase, ContentPhase::Loading);
    }

    #[test]
    fn phase_error_when_error_flag_set() {
        let phase = ContentPhase::from_lifecycle(false, true, true, false, true);
        assert_eq!(phase, ContentPhase::Error);
    }

    #[test]
    fn phase_empty_when_no_results_after_search() {
        let phase = ContentPhase::from_lifecycle(false, false, true, false, false);
        assert_eq!(phase, ContentPhase::Empty);
    }

    #[test]
    fn phase_loaded_when_results_exist() {
        let phase = ContentPhase::from_lifecycle(false, false, true, false, true);
        assert_eq!(phase, ContentPhase::Loaded);
    }

    #[test]
    fn phase_download_only_in_download_mode() {
        let phase = ContentPhase::from_lifecycle(false, false, true, true, false);
        assert_eq!(phase, ContentPhase::DownloadOnly);
    }
}
