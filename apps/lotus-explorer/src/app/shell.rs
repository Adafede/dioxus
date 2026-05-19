// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::draw_page::DrawPage;
use super::view::AppView;
use crate::app_state::{AppState, DownloadState};
use crate::components::data_curation_page::DataCurationPage;
use crate::components::layout::footer::Footer;
use crate::components::layout::header_meta::HeaderMetaSection;
use crate::components::layout::notices::{ErrorNotice, ShareNotice, TaxonNotice};
use crate::components::layout::page_header::PageHeader;
use crate::components::layout::sidebar::Sidebar;
use crate::components::results_viewport::ResultsViewport;
use crate::features::explore::actions::ExploreAction;
use crate::features::explore::download_dispatch::{
    use_download_dispatch_effect, use_startup_effect,
};
use crate::features::explore::orchestrator::SearchTaskController;
use crate::features::explore::search_state::{ExploreState, dispatch_explore_action};
use crate::features::explore::state::controller::{
    classes_for_view, retry_search, start_interactive_search, start_preview_search,
};
use crate::features::explore::url_state::{
    build_shareable_url, initial_url_state, persist_locale_query_param, persist_view_query_param,
};
use crate::hooks::LocaleProvider;
use crate::i18n::{Locale, TextKey, t};
use crate::models::SearchCriteria;
use crate::repositories::HybridRepository;
use crate::state::{
    AppStateContext, FormCriteriaContext, ResultsContext, use_form_criteria_context,
};
use crate::ui::a11y_contract::{MAIN_PANEL_ID, PAGE_TITLE_ID, SKIP_TO_RESULTS_HREF};
use dioxus::prelude::*;
use std::sync::Arc;

#[derive(Clone, Copy)]
struct AppDependencies {
    repo: HybridRepository,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn locale_lang_tag(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "en",
        Locale::Fr => "fr",
        Locale::De => "de",
        Locale::It => "it",
    }
}

#[cfg(target_arch = "wasm32")]
fn sync_document_lang(locale: Locale) {
    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
        && let Some(root) = document.document_element()
    {
        let _ = root.set_attribute("lang", locale_lang_tag(locale));
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn sync_document_lang(_: Locale) {}

#[component]
pub fn AppRoot() -> Element {
    let startup = initial_url_state();
    let initial_criteria_for_baseline = startup.criteria.clone();

    let app_state: Signal<AppState> = use_signal(|| AppState {
        view: startup.view,
        download: DownloadState {
            pending_format: startup.download.pending_format,
            pending_invalid_format: startup.download.pending_invalid_format,
            direct_execute: startup.download.direct_execute,
        },
        ..AppState::default()
    });
    let criteria: Signal<SearchCriteria> = use_signal(move || startup.criteria.clone());
    let criteria_baseline: Signal<SearchCriteria> =
        use_signal(move || initial_criteria_for_baseline.clone());
    let locale: Signal<Locale> = use_signal(|| startup.locale);
    let explore: Signal<ExploreState> = use_signal(ExploreState::default);

    let _deps = use_context_provider(|| AppDependencies {
        repo: HybridRepository::new(),
    });
    let _app_state_ctx = use_context_provider(move || AppStateContext::new(app_state));
    let _form_criteria_ctx =
        use_context_provider(move || FormCriteriaContext::new(criteria, criteria_baseline));
    let _results_ctx = use_context_provider(move || ResultsContext::new(explore));
    let search_task_controller = use_context_provider(SearchTaskController::new);

    let locale_value = *locale.read();
    let deps = use_context::<AppDependencies>();
    let repo = deps.repo;

    let shareable_url =
        use_memo(move || build_shareable_url(&criteria.read()).map(Arc::<str>::from));

    use_effect(move || persist_locale_query_param(*locale.read()));
    use_effect(move || persist_view_query_param(app_state.read().view));
    use_effect(move || sync_document_lang(*locale.read()));

    use_startup_effect(
        app_state,
        explore,
        criteria,
        search_task_controller.clone(),
        repo,
    );
    use_download_dispatch_effect(app_state, explore);

    let form_ctx = use_form_criteria_context();
    let on_search = {
        let tc = search_task_controller.clone();
        move |_: ()| {
            start_interactive_search(criteria, explore, tc.clone(), repo, form_ctx);
        }
    };
    let on_preview = {
        let tc = search_task_controller.clone();
        move |_: ()| start_preview_search(criteria, explore, tc.clone(), repo)
    };
    let tc_retry = search_task_controller.clone();

    let current_view = app_state.read().view;
    let layout_classes = classes_for_view(current_view);

    rsx! {
        LocaleProvider { locale,
            a { class: "skip-link", href: SKIP_TO_RESULTS_HREF,
                "{t(locale_value, TextKey::SkipToResults)}"
            }
            div { class: "{layout_classes.app_layout}",
                if current_view == AppView::Explore {
                    Sidebar { on_search }
                }

                main {
                    id: MAIN_PANEL_ID,
                    class: "{layout_classes.main}",
                    tabindex: "-1",
                    aria_labelledby: PAGE_TITLE_ID,
                    PageHeader {}

                    if current_view == AppView::Explore {
                        ShareNotice { shareable_url }
                        TaxonNotice {}
                        ErrorNotice {
                            on_dismiss: move |_| dispatch_explore_action(explore, ExploreAction::ErrorDismissed),
                            on_retry: move |_| retry_search(criteria, explore, tc_retry.clone(), repo),
                        }
                        HeaderMetaSection {}
                        ResultsViewport { on_preview }
                    } else if current_view == AppView::Curation {
                        DataCurationPage {}
                    } else {
                        DrawPage {}
                    }

                    Footer {}
                }
            }
        }
    }
}
