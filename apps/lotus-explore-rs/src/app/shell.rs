// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::bootstrap::{AppBootstrap, bootstrap_app};
use super::view::AppView;
use crate::app_state::AppState;
use crate::components::data_curation_page::DataCurationPage;
use crate::components::layout::footer::Footer;
use crate::components::layout::header_meta::HeaderMetaSection;
use crate::components::layout::notices::{ErrorNotice, ShareNotice, TaxonNotice};
use crate::components::layout::page_header::PageHeader;
use crate::components::layout::sidebar::Sidebar;
use crate::components::results_viewport::ResultsViewport;
use crate::document_head::{LotusDocumentHead, ToastTemplate};
use crate::features::explore::{
    ExploreInteractions, ExploreState, SearchTaskController, build_shareable_url,
    initial_url_state, persist_locale_query_param, persist_view_query_param,
    use_download_dispatch_effect, use_startup_effect,
};
use crate::hooks::LocaleProvider;
use crate::i18n::{Locale, TextKey, t};
use crate::models::SearchCriteria;
use crate::pages::DrawPage;
use crate::services::AppServices;
use crate::state::{
    AppStateContext, FormCriteriaContext, ResultsContext, use_app_selector, use_app_state_context,
    use_form_criteria_context,
};
use crate::ui::a11y_contract::{MAIN_PANEL_ID, PAGE_TITLE_ID, SKIP_TO_RESULTS_HREF};
use dioxus::prelude::*;
use std::sync::Arc;
use ui::prelude::*;

const fn locale_lang_tag(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "en",
        Locale::Fr => "fr",
        Locale::De => "de",
        Locale::It => "it",
    }
}

fn resolve_startup_dark_mode(startup: &crate::features::explore::InitialUrlState) -> bool {
    if startup.dark_mode {
        return true;
    }

    #[cfg(target_arch = "wasm32")]
    {
        let params = crate::features::explore::url_state::read_url_query_params();
        if let Some(raw) = params.get("dark_mode") {
            return crate::features::explore::url_state::is_true_flag(raw);
        }

        if let Some(win) = web_sys::window() {
            if let Ok(media) = win.match_media("(prefers-color-scheme: dark)") {
                if let Some(media_query) = media {
                    return media_query.matches();
                }
            }
        }
    }

    false
}

#[component]
pub fn AppRoot() -> Element {
    let startup_url_state = initial_url_state();
    let startup_dark_mode = resolve_startup_dark_mode(&startup_url_state);
    let AppBootstrap {
        app_state: initial_app_state,
        criteria: initial_criteria,
        criteria_baseline: initial_criteria_baseline,
        locale: initial_locale,
        explore: initial_explore,
    } = bootstrap_app(startup_url_state);

    let app_state: Signal<AppState> = use_signal(move || AppState {
        dark_mode: startup_dark_mode,
        ..initial_app_state.clone()
    });
    let criteria: Signal<SearchCriteria> = use_signal(move || initial_criteria);
    let criteria_baseline: Signal<SearchCriteria> = use_signal(move || initial_criteria_baseline);
    let locale: Signal<Locale> = use_signal(move || initial_locale);
    let explore: Signal<ExploreState> = use_signal(move || initial_explore);

    let services = use_context_provider(AppServices::new);
    let repo = services.repository();
    let _app_state_ctx = use_context_provider(move || AppStateContext::new(app_state));
    let form_ctx =
        use_context_provider(move || FormCriteriaContext::new(criteria, criteria_baseline));
    let _results_ctx = use_context_provider(move || ResultsContext::new(explore));
    let search_task_controller = use_context_provider(SearchTaskController::new);
    let _explore_interactions = use_context_provider({
        let tc = search_task_controller;
        move || ExploreInteractions::new(criteria, form_ctx, explore, tc, repo)
    });

    rsx! {
        LocaleProvider { locale,
            AppRuntimeEffects {
                app_state,
                explore,
                criteria,
            }
            ShellScaffold { lang: locale_lang_tag(*locale.read()).to_string() }
        }
    }
}

#[component]
fn AppRuntimeEffects(
    app_state: Signal<AppState>,
    explore: Signal<ExploreState>,
    criteria: Signal<SearchCriteria>,
) -> Element {
    let locale = crate::hooks::use_locale_signal();
    let search_task_controller = use_context::<SearchTaskController>();
    let repo = use_context::<AppServices>().repository();

    use_effect(move || persist_locale_query_param(*locale.read()));
    use_effect(move || persist_view_query_param(app_state.read().view));
    use_effect(move || {
        let dark_mode = app_state.read().dark_mode;
        #[cfg(target_arch = "wasm32")]
        {
            let params = crate::features::explore::url_state::read_url_query_params();
            let effective_dark_mode = params
                .get("dark_mode")
                .map(|raw| crate::features::explore::url_state::is_true_flag(raw))
                .unwrap_or(dark_mode);

            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(html) = doc.document_element() {
                    if effective_dark_mode {
                        let _ = html.set_attribute("data-theme", "dark");
                    } else {
                        let _ = html.set_attribute("data-theme", "light");
                    }
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = dark_mode;
        }
    });

    use_startup_effect(app_state, explore, criteria, search_task_controller, repo);
    use_download_dispatch_effect(app_state, explore);

    rsx! {}
}

#[component]
fn ShellScaffold(lang: String) -> Element {
    let locale = crate::hooks::use_locale();
    let app_state = use_app_state_context().state;
    let current_view = *use_app_selector(app_state, |state| state.view).read();
    let single_pane = current_view != AppView::Explore;

    rsx! {
        LotusDocumentHead { lang }
        ToastTemplate {}
        a { class: "skip-link", href: SKIP_TO_RESULTS_HREF, style: skip_link_style(),
            "{t(locale, TextKey::SkipToResults)}"
        }
        div { class: "app-shell",
            div { class: "app-layout",
                if current_view == AppView::Explore {
                    Sidebar {}
                }

                main {
                    id: MAIN_PANEL_ID,
                    class: if single_pane { "main-content single-pane" } else { "main-content" },
                    tabindex: "-1",
                    aria_labelledby: PAGE_TITLE_ID,
                    PageHeader {}
                    RouteContent { current_view }
                }
            }
            Footer {}
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

fn skip_link_style() -> String {
    StyleBuilder::new()
        .property("position", "absolute")
        .property("left", "0.5rem")
        .property("top", "-100%")
        .property("z-index", "9999")
        .padding("0.5rem 1rem")
        .background_color("transparent")
        .color("var(--text)")
        .font_size("0.875rem")
        .font_weight("600")
        .border_radius("0 0 4px 4px")
        .text_decoration("none")
        .property("transition", "top 0.1s")
        .build()
}
