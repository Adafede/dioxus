// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::app::view::AppView;
use crate::features::explore::command::SearchCommand;
use crate::features::explore::orchestrator::{SearchTaskController, start_search};
use crate::features::explore::search_state::ExploreState;
use crate::models::SearchCriteria;
use crate::repositories::LotusRepository;
use crate::state::FormCriteriaContext;
use dioxus::prelude::Signal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppLayoutClasses {
    pub app_layout: &'static str,
    pub main: &'static str,
}

pub fn classes_for_view(view: AppView) -> AppLayoutClasses {
    if view == AppView::Explore {
        AppLayoutClasses {
            app_layout: "app-layout",
            main: "main-content",
        }
    } else {
        AppLayoutClasses {
            app_layout: "app-layout no-sidebar",
            main: "main-content single-pane",
        }
    }
}

pub fn start_interactive_search<R: LotusRepository>(
    criteria: Signal<SearchCriteria>,
    explore: Signal<ExploreState>,
    task_controller: SearchTaskController,
    repo: R,
    form_ctx: FormCriteriaContext,
) {
    form_ctx.mark_searched();
    start_search(
        criteria,
        SearchCommand::Interactive,
        explore,
        task_controller,
        repo,
    );
}

pub fn start_preview_search<R: LotusRepository>(
    criteria: Signal<SearchCriteria>,
    explore: Signal<ExploreState>,
    task_controller: SearchTaskController,
    repo: R,
) {
    start_search(
        criteria,
        SearchCommand::Interactive,
        explore,
        task_controller,
        repo,
    );
}

pub fn retry_search<R: LotusRepository>(
    criteria: Signal<SearchCriteria>,
    explore: Signal<ExploreState>,
    task_controller: SearchTaskController,
    repo: R,
) {
    start_search(
        criteria,
        SearchCommand::Interactive,
        explore,
        task_controller,
        repo,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_for_explore_view_keeps_sidebar_layout() {
        let classes = classes_for_view(AppView::Explore);
        assert_eq!(classes.app_layout, "app-layout");
        assert_eq!(classes.main, "main-content");
    }

    #[test]
    fn classes_for_non_explore_view_uses_single_pane_layout() {
        let classes = classes_for_view(AppView::Curation);
        assert_eq!(classes.app_layout, "app-layout no-sidebar");
        assert_eq!(classes.main, "main-content single-pane");
    }
}
