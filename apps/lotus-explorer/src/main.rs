// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

#![allow(non_snake_case)]

mod api;
mod app;
mod app_state;
mod components;
mod curation;
mod download;
mod export;
mod features;
mod hooks;
mod i18n;
mod models;
mod perf;
mod queries;
mod repositories;
mod sparql;
mod state;
mod utils;

use app::draw_page::DrawPage;
use app::view::AppView;
use app_state::{AppState, DownloadState, SearchState, UiState};
use components::layout::footer::Footer;
use components::layout::header_meta::HeaderMetaSection;
use components::layout::notices::{ErrorNotice, ShareNotice, TaxonNotice};
use components::results_viewport::ResultsViewport;
use components::search_panel::SearchPanel;
use dioxus::prelude::*;
#[cfg(test)]
use download::DownloadFormat;
use features::explore::actions::ExploreAction;
use features::explore::download_dispatch::{use_download_dispatch_effect, use_startup_effect};
use features::explore::orchestrator::start_search;
use features::explore::search_state::{ExploreState, dispatch_explore_action};
use features::explore::url_state::{
    build_shareable_url, initial_criteria_from_url, initial_download_format_from_url,
    initial_execute_from_url, initial_locale_from_url, initial_view_from_url,
    persist_locale_query_param, persist_view_query_param,
};
use hooks::LocaleProvider;
use i18n::{
    Locale, TextKey, t, view_label_curation_explorer, view_label_draw, view_label_explorer,
    view_switch_aria,
};
use models::*;
use repositories::HybridRepository;
use state::{AppStateContext, FormCriteriaContext, ResultsContext, SearchUiContext};
use std::sync::Arc;

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
    let initial_criteria = initial_criteria_from_url();
    let mut app_state: Signal<AppState> = use_signal(|| AppState {
        view: initial_view_from_url(),
        search: SearchState {
            criteria: initial_criteria.clone(),
            ..SearchState::default()
        },
        ui: UiState {
            locale: initial_locale_from_url(),
            ..UiState::default()
        },
        download: DownloadState {
            pending_format: initial_download_format_from_url(),
            direct_execute: initial_execute_from_url(),
        },
        ..AppState::default()
    });
    let criteria: Signal<SearchCriteria> = use_signal(move || initial_criteria.clone());
    let mut locale: Signal<Locale> = use_signal(initial_locale_from_url);
    let explore: Signal<ExploreState> = use_signal(ExploreState::default);

    let locale_value = *locale.read();
    let mobile_filters_open = explore.read().ui.mobile_filters_open;
    let repo = HybridRepository::new();

    let _app_state_ctx = use_context_provider(move || AppStateContext::new(app_state));
    let _search_ui_ctx =
        use_context_provider(move || SearchUiContext::from_signals(app_state, criteria));
    let _form_criteria_ctx = use_context_provider(move || FormCriteriaContext::new(criteria));
    let _results_ctx =
        use_context_provider(move || ResultsContext::from_signals(app_state, explore));

    let shareable_url =
        use_memo(move || build_shareable_url(&criteria.read()).map(Arc::<str>::from));

    use_effect(move || {
        persist_locale_query_param(*locale.read());
    });
    use_effect(move || {
        if app_state.read().ui.locale != *locale.read() {
            app_state.with_mut(|state| state.ui.locale = *locale.read());
        }
    });
    use_effect(move || {
        let criteria_snapshot = criteria.read().clone();
        if app_state.read().search.criteria != criteria_snapshot {
            app_state.with_mut(|state| state.search.criteria = criteria_snapshot);
        }
    });
    use_effect(move || {
        let explore_snapshot = explore.read().clone();
        let mobile_filters_open = explore_snapshot.ui.mobile_filters_open;
        if app_state.read().search.explore != explore_snapshot {
            app_state.with_mut(|state| state.search.explore = explore_snapshot);
        }
        if app_state.read().ui.mobile_filters_open != mobile_filters_open {
            app_state.with_mut(|state| state.ui.mobile_filters_open = mobile_filters_open);
        }
    });
    use_effect(move || {
        persist_view_query_param(app_state.read().view);
    });

    use_startup_effect(app_state, explore, criteria, repo);
    use_download_dispatch_effect(app_state, explore);

    let on_search = move |_: ()| start_search(criteria, false, explore, repo);
    let on_preview = move |_: ()| start_search(criteria, false, explore, repo);

    let app_layout_class = if app_state.read().view == AppView::Explore {
        "app-layout"
    } else {
        "app-layout no-sidebar"
    };

    let main_class = if app_state.read().view == AppView::Explore {
        "main-content"
    } else {
        "main-content single-pane"
    };

    rsx! {
        LocaleProvider { locale,
            a { class: "skip-link", href: "#main-panel", "{t(locale_value, TextKey::SkipToResults)}" }
            div { class: "{app_layout_class}",

                if app_state.read().view == AppView::Explore {
                    aside {
                        class: if mobile_filters_open { "sidebar mobile-open" } else { "sidebar mobile-closed" },
                        button {
                            class: "filters-toggle",
                            r#type: "button",
                            aria_pressed: if mobile_filters_open { "true" } else { "false" },
                            onclick: move |_| dispatch_explore_action(explore, ExploreAction::MobileFiltersToggled),
                            if mobile_filters_open {
                                "{t(locale_value, TextKey::FiltersHide)}"
                            } else {
                                "{t(locale_value, TextKey::FiltersShow)}"
                            }
                        }
                        SearchPanel { on_search }
                        div { class: "sidebar-logo-wrap",
                            img {
                                class: "sidebar-logo",
                                src: "assets/lotus_ferris.svg",
                                alt: "{t(locale_value, TextKey::PageTitle)}",
                            }
                        }
                    }
                }

                main { id: "main-panel", class: "{main_class}", tabindex: "-1",

                div { class: "page-header",
                    div { class: "page-brand",
                        h1 { class: "page-title",
                            a {
                                class: "page-title-link",
                                href: "?",
                                title: "{t(locale_value, TextKey::PageTitle)}",
                                aria_label: "{t(locale_value, TextKey::PageTitle)}",
                                "{t(locale_value, TextKey::PageTitle)}"
                            }
                        }
                        div {
                            class: "lang-switch",
                            role: "group",
                            aria_label: "{t(locale_value, TextKey::Language)}",
                            button {
                                class: if locale_value == Locale::En { "btn btn-xs lang-btn active" } else { "btn btn-xs lang-btn" },
                                r#type: "button",
                                aria_pressed: if locale_value == Locale::En { "true" } else { "false" },
                                onclick: move |_| {
                                    if *locale.peek() != Locale::En {
                                        *locale.write() = Locale::En;
                                    }
                                },
                                "EN"
                            }
                            button {
                                class: if locale_value == Locale::Fr { "btn btn-xs lang-btn active" } else { "btn btn-xs lang-btn" },
                                r#type: "button",
                                aria_pressed: if locale_value == Locale::Fr { "true" } else { "false" },
                                onclick: move |_| {
                                    if *locale.peek() != Locale::Fr {
                                        *locale.write() = Locale::Fr;
                                    }
                                },
                                "FR"
                            }
                            button {
                                class: if locale_value == Locale::De { "btn btn-xs lang-btn active" } else { "btn btn-xs lang-btn" },
                                r#type: "button",
                                aria_pressed: if locale_value == Locale::De { "true" } else { "false" },
                                onclick: move |_| {
                                    if *locale.peek() != Locale::De {
                                        *locale.write() = Locale::De;
                                    }
                                },
                                "DE"
                            }
                            button {
                                class: if locale_value == Locale::It { "btn btn-xs lang-btn active" } else { "btn btn-xs lang-btn" },
                                r#type: "button",
                                aria_pressed: if locale_value == Locale::It { "true" } else { "false" },
                                onclick: move |_| {
                                    if *locale.peek() != Locale::It {
                                        *locale.write() = Locale::It;
                                    }
                                },
                                "IT"
                            }
                        }
                    }
                    nav {
                        class: "view-switch",
                        aria_label: "{view_switch_aria(locale_value)}",
                        button {
                            class: if app_state.read().view == AppView::Explore { "btn btn-xs lang-btn active" } else { "btn btn-xs lang-btn" },
                            r#type: "button",
                            aria_pressed: if app_state.read().view == AppView::Explore { "true" } else { "false" },
                            onclick: move |_| app_state.with_mut(|state| state.view = AppView::Explore),
                            "{view_label_explorer(locale_value)}"
                        }
                        button {
                            class: if app_state.read().view == AppView::Curation { "btn btn-xs lang-btn active" } else { "btn btn-xs lang-btn" },
                            r#type: "button",
                            aria_pressed: if app_state.read().view == AppView::Curation { "true" } else { "false" },
                            onclick: move |_| app_state.with_mut(|state| state.view = AppView::Curation),
                            "{view_label_curation_explorer(locale_value)}"
                        }
                        button {
                            class: if app_state.read().view == AppView::Draw { "btn btn-xs lang-btn active" } else { "btn btn-xs lang-btn" },
                            r#type: "button",
                            aria_pressed: if app_state.read().view == AppView::Draw { "true" } else { "false" },
                            onclick: move |_| app_state.with_mut(|state| state.view = AppView::Draw),
                            "{view_label_draw(locale_value)}"
                        }
                    }
                    p { class: "page-sub", "{t(locale_value, TextKey::PageSubtitle)}" }
                    p { class: "page-archive-note",
                        span { class: "page-archive-label", "{t(locale_value, TextKey::ArchiveNotice)}" }
                        a {
                            class: "page-archive-link mono",
                            href: "https://doi.org/10.5281/zenodo.5794106",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "10.5281/zenodo.5794106"
                        }
                    }
                }

                if app_state.read().view == AppView::Explore {
                    HeaderMetaSection { explore, locale }
                    ShareNotice { locale, shareable_url }
                    TaxonNotice { explore, locale }
                    ErrorNotice {
                        explore,
                        locale,
                        on_dismiss: move |_| dispatch_explore_action(explore, ExploreAction::ErrorDismissed),
                        on_retry: move |_| start_search(criteria, false, explore, repo),
                    }
                    ResultsViewport { on_preview }
                } else if app_state.read().view == AppView::Curation {
                    components::data_curation_page::DataCurationPage { locale: locale_value }
                } else {
                    DrawPage { locale: locale_value }
                }

                    Footer { locale: locale_value }
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
