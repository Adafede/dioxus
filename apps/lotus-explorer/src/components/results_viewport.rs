// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Top-level results area component using phase-driven rendering.

use crate::components::loading::{DownloadDispatchState, DownloadOnlyState, LoadingState};
use crate::components::results_table::ResultsTable;
use crate::components::welcome::WelcomeScreen;
use crate::state::use_results_context;
use crate::ui::ContentPhase;
use dioxus::prelude::*;

#[component]
pub fn ResultsViewport(on_preview: EventHandler<()>) -> Element {
    use crate::features::explore::selectors::{use_lifecycle_selector, use_result_selector};

    let state = use_results_context();
    let explore = state.explore;
    // Hoisted to component top-level — hooks must be called unconditionally.
    let locale = crate::hooks::use_locale();

    let loading = use_lifecycle_selector(explore, |lc| lc.loading);
    let has_error = use_lifecycle_selector(explore, |lc| lc.error.is_some());
    let searched_once = use_lifecycle_selector(explore, |lc| lc.searched_once);
    let download_only_mode = use_lifecycle_selector(explore, |lc| lc.download_only_mode);
    let download_dispatching = use_lifecycle_selector(explore, |lc| lc.download_dispatching);
    let entries_empty = use_result_selector(explore, |r| r.entries.is_empty());

    let phase = use_memo(move || {
        ContentPhase::from_lifecycle(
            *loading.read(),
            *has_error.read(),
            *searched_once.read(),
            *download_only_mode.read(),
            !*entries_empty.read(),
        )
    });

    match *phase.read() {
        ContentPhase::Welcome => rsx! {
            WelcomeScreen {}
        },
        ContentPhase::Loading => rsx! {
            LoadingState {}
        },
        // Error state: `ErrorNotice` (rendered above this viewport in the page
        // layout) already shows the full typed error with dismiss + retry
        // actions.  Rendering a second error message here would be redundant
        // and could clash with localised notice text, so we intentionally yield
        // an empty fragment and let the notice carry the UX weight.
        ContentPhase::Error => rsx! {},
        ContentPhase::Empty => {
            rsx! {
                div { class: "empty-state",
                    p { class: "form-hint", "{crate::i18n::t(locale, crate::i18n::TextKey::NoResults)}" }
                }
            }
        }
        ContentPhase::Loaded => rsx! {
            ResultsTable {}
        },
        ContentPhase::DownloadOnly => {
            if *download_dispatching.read() {
                rsx! {
                    DownloadDispatchState {}
                }
            } else {
                rsx! {
                    DownloadOnlyState { on_preview }
                }
            }
        }
    }
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
