// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::bootstrap::{AppBootstrap, bootstrap_app};
use super::draw_page::DrawPage;
use super::view::AppView;
use crate::app_state::AppState;
use crate::components::data_curation_page::DataCurationPage;
use crate::components::layout::footer::Footer;
use crate::components::layout::header_meta::HeaderMetaSection;
use crate::components::layout::notices::{ErrorNotice, ShareNotice, TaxonNotice};
use crate::components::layout::page_header::PageHeader;
use crate::components::layout::sidebar::Sidebar;
use crate::components::results_viewport::ResultsViewport;
use crate::features::explore::{
    ExploreInteractions, ExploreState, SearchTaskController, build_shareable_url, classes_for_view,
    initial_url_state, persist_locale_query_param, persist_view_query_param,
    use_download_dispatch_effect, use_startup_effect,
};
use crate::hooks::LocaleProvider;
use crate::i18n::{Locale, TextKey, t};
use crate::models::SearchCriteria;
use crate::services::AppServices;
use crate::state::{
    AppStateContext, FormCriteriaContext, ResultsContext, use_app_selector, use_app_state_context,
    use_form_criteria_context,
};
use crate::ui::a11y_contract::{MAIN_PANEL_ID, PAGE_TITLE_ID, SKIP_TO_RESULTS_HREF};
use dioxus::prelude::*;
use std::sync::Arc;

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
fn sync_document_lang(locale: Locale) {
    let _ = locale_lang_tag(locale);
}

#[component]
pub fn AppRoot() -> Element {
    let AppBootstrap {
        app_state: initial_app_state,
        criteria: initial_criteria,
        criteria_baseline: initial_criteria_baseline,
        locale: initial_locale,
        explore: initial_explore,
    } = bootstrap_app(initial_url_state());

    let app_state: Signal<AppState> = use_signal(move || initial_app_state.clone());
    let criteria: Signal<SearchCriteria> = use_signal(move || initial_criteria.clone());
    let criteria_baseline: Signal<SearchCriteria> =
        use_signal(move || initial_criteria_baseline.clone());
    let locale: Signal<Locale> = use_signal(move || initial_locale);
    let explore: Signal<ExploreState> = use_signal(move || initial_explore.clone());

    let services = use_context_provider(AppServices::new);
    let repo = services.repository();
    let _app_state_ctx = use_context_provider(move || AppStateContext::new(app_state));
    let form_ctx =
        use_context_provider(move || FormCriteriaContext::new(criteria, criteria_baseline));
    let _results_ctx = use_context_provider(move || ResultsContext::new(explore));
    let search_task_controller = use_context_provider(SearchTaskController::new);
    let _explore_interactions = use_context_provider({
        let tc = search_task_controller.clone();
        move || ExploreInteractions::new(criteria, form_ctx, explore, tc, repo)
    });

    rsx! {
        LocaleProvider { locale,
            AppRuntimeEffects {
                app_state,
                explore,
                criteria,
                locale,
            }
            ShellScaffold {}
        }
    }
}

#[component]
fn AppRuntimeEffects(
    app_state: Signal<AppState>,
    explore: Signal<ExploreState>,
    criteria: Signal<SearchCriteria>,
    locale: Signal<Locale>,
) -> Element {
    let search_task_controller = use_context::<SearchTaskController>();
    let repo = use_context::<AppServices>().repository();

    use_effect(move || persist_locale_query_param(*locale.read()));
    use_effect(move || persist_view_query_param(app_state.read().view));
    use_effect(move || sync_document_lang(*locale.read()));

    use_startup_effect(app_state, explore, criteria, search_task_controller, repo);
    use_download_dispatch_effect(app_state, explore);

    rsx! {}
}

#[component]
fn ShellScaffold() -> Element {
    let locale = crate::hooks::use_locale();
    let app_state = use_app_state_context().state;
    let current_view = *use_app_selector(app_state, |state| state.view).read();
    let layout_classes = classes_for_view(current_view);

    rsx! {
        a { class: "skip-link", href: SKIP_TO_RESULTS_HREF,
            "{t(locale, TextKey::SkipToResults)}"
        }
        div { class: "{layout_classes.app_layout}",
            if current_view == AppView::Explore {
                Sidebar {}
            }

            main {
                id: MAIN_PANEL_ID,
                class: "{layout_classes.main}",
                tabindex: "-1",
                aria_labelledby: PAGE_TITLE_ID,
                PageHeader {}
                RouteContent { current_view }
                Footer {}
            }
        }
    }
}

#[component]
fn RouteContent(current_view: AppView) -> Element {
    match current_view {
        AppView::Explore => rsx! { ExplorePage {} },
        AppView::Curation => rsx! { DataCurationPage {} },
        AppView::Draw => rsx! { DrawPage {} },
    }
}

#[component]
fn ExplorePage() -> Element {
    let criteria = use_form_criteria_context().criteria;
    let shareable_url =
        use_memo(move || build_shareable_url(&criteria.read()).map(Arc::<str>::from));

    rsx! {
        ShareNotice { shareable_url }
        TaxonNotice {}
        ErrorNotice {}
        HeaderMetaSection {}
        ResultsViewport {}
    }
}
