// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

#![allow(non_snake_case)]

mod api;
mod app;
mod app_state;
mod components;
mod core;
mod curation;
mod data;
mod download;
mod export;
mod features;
mod hooks;
mod i18n;
mod models;
mod perf;
mod queries;
mod repositories;
mod services;
mod sparql;
mod state;
mod ui;
mod utils;

use app::draw_page::DrawPage;
use app::view::AppView;
use app_state::{AppState, DownloadState};
use components::layout::footer::Footer;
use components::layout::header_meta::HeaderMetaSection;
use components::layout::notices::{ErrorNotice, ShareNotice, TaxonNotice};
use components::layout::page_header::PageHeader;
use components::layout::sidebar::Sidebar;
use components::results_viewport::ResultsViewport;
use dioxus::prelude::*;
#[cfg(test)]
use download::DownloadFormat;
use features::explore::actions::ExploreAction;
use features::explore::command::SearchCommand;
use features::explore::download_dispatch::{use_download_dispatch_effect, use_startup_effect};
use features::explore::orchestrator::{SearchTaskController, start_search};
use features::explore::search_state::{ExploreState, dispatch_explore_action};
use features::explore::url_state::{
    build_shareable_url, initial_url_state, persist_locale_query_param, persist_view_query_param,
};
use hooks::LocaleProvider;
use i18n::{Locale, TextKey, t};
use models::*;
use repositories::HybridRepository;
use state::{AppStateContext, FormCriteriaContext, ResultsContext, use_form_criteria_context};
use std::sync::Arc;
use ui::a11y_contract::{MAIN_PANEL_ID, PAGE_TITLE_ID, SKIP_TO_RESULTS_HREF};

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
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(root) = document.document_element() {
                let _ = root.set_attribute("lang", locale_lang_tag(locale));
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn sync_document_lang(_: Locale) {}

fn main() {
    let level = if cfg!(debug_assertions) {
        log::Level::Debug
    } else {
        log::Level::Info
    };
    console_log::init_with_level(level).ok();
    launch(App);
}

#[component]
fn App() -> Element {
    // ── Initialise from URL parameters ────────────────────────────────────────
    let startup = initial_url_state();
    let initial_view = startup.view;
    let initial_locale = startup.locale;
    let initial_download = startup.download;
    let initial_criteria = startup.criteria;
    let initial_criteria_for_baseline = initial_criteria.clone();

    // ── Canonical signals (each is sole owner of its data) ────────────────────
    let app_state: Signal<AppState> = use_signal(|| AppState {
        view: initial_view,
        download: DownloadState {
            pending_format: initial_download.pending_format,
            pending_invalid_format: initial_download.pending_invalid_format,
            direct_execute: initial_download.direct_execute,
        },
        ..AppState::default()
    });
    let criteria: Signal<SearchCriteria> = use_signal(move || initial_criteria.clone());
    let criteria_baseline: Signal<SearchCriteria> =
        use_signal(move || initial_criteria_for_baseline.clone());
    let locale: Signal<Locale> = use_signal(|| initial_locale);
    let explore: Signal<ExploreState> = use_signal(ExploreState::default);

    let locale_value = *locale.read();
    let repo = HybridRepository::new();

    // ── Context providers ─────────────────────────────────────────────────────
    let _app_state_ctx = use_context_provider(move || AppStateContext::new(app_state));
    let _form_criteria_ctx =
        use_context_provider(move || FormCriteriaContext::new(criteria, criteria_baseline));
    let _results_ctx = use_context_provider(move || ResultsContext::new(explore));
    let search_task_controller = use_context_provider(SearchTaskController::new);

    // ── Shareable URL ─────────────────────────────────────────────────────────
    let shareable_url =
        use_memo(move || build_shareable_url(&criteria.read()).map(Arc::<str>::from));

    // ── Persistent side-effects ───────────────────────────────────────────────
    use_effect(move || persist_locale_query_param(*locale.read()));
    use_effect(move || persist_view_query_param(app_state.read().view));
    use_effect(move || sync_document_lang(*locale.read()));

    // ── Feature hooks ─────────────────────────────────────────────────────────
    use_startup_effect(
        app_state,
        explore,
        criteria,
        search_task_controller.clone(),
        repo,
    );
    use_download_dispatch_effect(app_state, explore);

    // ── Event handlers (capture App-scope values) ─────────────────────────────
    let form_ctx = use_form_criteria_context();
    let on_search = {
        let tc = search_task_controller.clone();
        move |_: ()| {
            form_ctx.mark_searched();
            start_search(
                criteria,
                SearchCommand::Interactive,
                explore,
                tc.clone(),
                repo,
            );
        }
    };
    let on_preview = {
        let tc = search_task_controller.clone();
        move |_: ()| {
            start_search(
                criteria,
                SearchCommand::Interactive,
                explore,
                tc.clone(),
                repo,
            )
        }
    };
    let tc_retry = search_task_controller.clone();

    // ── Layout ────────────────────────────────────────────────────────────────
    let current_view = app_state.read().view;
    let app_layout_class = if current_view == AppView::Explore {
        "app-layout"
    } else {
        "app-layout no-sidebar"
    };
    let main_class = if current_view == AppView::Explore {
        "main-content"
    } else {
        "main-content single-pane"
    };

    rsx! {
        LocaleProvider { locale,
            a { class: "skip-link", href: SKIP_TO_RESULTS_HREF,
                "{t(locale_value, TextKey::SkipToResults)}"
            }
            div { class: "{app_layout_class}",

                if current_view == AppView::Explore {
                    Sidebar { on_search }
                }

                main {
                    id: MAIN_PANEL_ID,
                    class: "{main_class}",
                    tabindex: "-1",
                    aria_labelledby: PAGE_TITLE_ID,
                    PageHeader {}

                    if current_view == AppView::Explore {
                        ShareNotice { shareable_url }
                        TaxonNotice {}
                        ErrorNotice {
                            on_dismiss: move |_| dispatch_explore_action(explore, ExploreAction::ErrorDismissed),
                            on_retry: move |_| start_search(criteria, SearchCommand::Interactive, explore, tc_retry.clone(), repo),
                        }
                        HeaderMetaSection {}
                        ResultsViewport { on_preview }
                    } else if current_view == AppView::Curation {
                        components::data_curation_page::DataCurationPage {}
                    } else {
                        DrawPage {}
                    }

                    Footer {}
                }
            }
        }
    }
}

#[cfg(test)]
fn is_supported_download_format(fmt: &str) -> bool {
    DownloadFormat::from_str(fmt).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_download_formats_include_documented_values() {
        assert!(is_supported_download_format("csv"));
        assert!(is_supported_download_format("json"));
        assert!(is_supported_download_format("ndjson"));
        assert!(is_supported_download_format("rdf"));
        assert!(!is_supported_download_format("ttl"));
    }
}
