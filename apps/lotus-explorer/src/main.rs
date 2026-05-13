// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

#![allow(non_snake_case)]

mod api;
mod app;
mod components;
mod curation;
mod download;
mod export;
mod features;
mod i18n;
mod models;
mod perf;
mod queries;
mod sparql;
mod state;
mod utils;

use app::draw_page::DrawPage;
use app::view::AppView;
use components::layout::footer::Footer;
use components::layout::header_meta::HeaderMetaSection;
use components::layout::notices::{ErrorNotice, ShareNotice, TaxonNotice};
use components::results_viewport::ResultsViewport;
use components::search_panel::SearchPanel;
use dioxus::prelude::*;
use download::{DownloadFormat, execute_download};
use features::explore::orchestrator::start_search;
use features::explore::search_state::{SearchRuntime, set_signal_if_changed};
use features::explore::types::{ErrorKind, QueryPhase};
use features::explore::url_state::{
    build_shareable_url, initial_criteria_from_url, initial_download_format_from_url,
    initial_execute_from_url, initial_locale_from_url, initial_view_from_url,
    persist_locale_query_param, persist_view_query_param,
};
use i18n::{
    Locale, TextKey, err_unsupported_format, t, view_label_curation_explorer, view_label_draw,
    view_label_explorer, view_switch_aria,
};
use models::*;
use state::{ResultsContext, SearchUiContext};
use std::sync::Arc;
use utils::logging::{log_debug_evt, log_info_evt, log_warn_evt};

fn main() {
    let level = if cfg!(debug_assertions) {
        log::Level::Debug
    } else {
        log::Level::Info
    };
    console_log::init_with_level(level).ok();
    launch(App);
}

// ── Root component ────────────────────────────────────────────────────────────

#[component]
fn App() -> Element {
    let mut app_view: Signal<AppView> = use_signal(initial_view_from_url);
    let criteria: Signal<SearchCriteria> = use_signal(initial_criteria_from_url);
    let executed_criteria: Signal<SearchCriteria> = use_signal(initial_criteria_from_url);
    let mut locale: Signal<Locale> = use_signal(initial_locale_from_url);
    // Entries live behind an `Arc<[…]>` so prop/signal clones are a single
    // refcount bump instead of duplicating the whole result buffer.
    let entries: Signal<Rows> = use_signal(|| Arc::<[CompoundEntry]>::from([]));
    let loading: Signal<bool> = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let error_kind: Signal<ErrorKind> = use_signal(|| ErrorKind::Unknown);
    let query_phase: Signal<QueryPhase> = use_signal(|| QueryPhase::Idle);
    let searched_once: Signal<bool> = use_signal(|| false);
    let download_only_mode: Signal<bool> = use_signal(|| false);
    let download_dispatching: Signal<bool> = use_signal(|| false);
    let taxon_notice: Signal<Option<String>> = use_signal(|| None);
    let resolved_qid: Signal<Option<String>> = use_signal(|| None);
    let query_hash: Signal<Option<String>> = use_signal(|| None);
    let result_hash: Signal<Option<String>> = use_signal(|| None);
    let sparql_query: Signal<Option<Arc<str>>> = use_signal(|| None);
    let metadata_json: Signal<Option<Arc<str>>> = use_signal(|| None);
    let total_matches: Signal<Option<usize>> = use_signal(|| None);
    let total_stats: Signal<Option<DatasetStats>> = use_signal(|| None);
    let display_capped_rows: Signal<bool> = use_signal(|| false);
    let sort: Signal<SortState> = use_signal(SortState::default);
    let mut mobile_filters_open: Signal<bool> = use_signal(|| false);
    let pending_download_format: Signal<Option<String>> =
        use_signal(initial_download_format_from_url);
    let pending_execute: Signal<bool> = use_signal(initial_execute_from_url);
    // Guard flags: never read in RSX — `.peek()` only — so they never create
    // reactive subscriptions for themselves.
    let waiting_loading_logged: Signal<bool> = use_signal(|| false);
    let waiting_query_logged: Signal<bool> = use_signal(|| false);
    let search_request_token: Signal<u64> = use_signal(|| 0);

    let locale_value = *locale.read();
    let mobile_open = *mobile_filters_open.read();

    let search_runtime = SearchRuntime {
        executed_criteria,
        loading,
        error,
        error_kind,
        query_phase,
        searched_once,
        download_only_mode,
        download_dispatching,
        entries,
        taxon_notice,
        resolved_qid,
        query_hash,
        result_hash,
        sparql_query,
        metadata_json,
        total_matches,
        total_stats,
        display_capped_rows,
        mobile_filters_open,
        search_request_token,
    };

    let _search_ui_ctx =
        use_context_provider(move || SearchUiContext::from_signals(criteria, locale, loading));
    let _results_ctx = use_context_provider(move || {
        ResultsContext::from_signals(
            executed_criteria,
            locale,
            entries,
            loading,
            error,
            query_phase,
            searched_once,
            download_only_mode,
            download_dispatching,
            query_hash,
            result_hash,
            sparql_query,
            metadata_json,
            total_matches,
            total_stats,
            display_capped_rows,
            sort,
        )
    });

    let shareable_url =
        use_memo(move || build_shareable_url(&criteria.read()).map(Arc::<str>::from));

    use_effect(move || {
        persist_locale_query_param(*locale.read());
    });
    use_effect(move || {
        persist_view_query_param(*app_view.read());
    });

    // ── Programmatic startup: auto-search / auto-download ─────────────────
    use_effect(move || {
        let pending = pending_download_format.read().clone();
        if let Some(fmt) = pending.as_deref()
            && DownloadFormat::from_str(fmt).is_none()
        {
            log_warn_evt(
                "download",
                "startup",
                "unsupported_format",
                Some(&format!("format={fmt}")),
            );
            set_signal_if_changed(error_kind, ErrorKind::Validation);
            set_signal_if_changed(error, Some(err_unsupported_format(*locale.peek(), fmt)));
            set_signal_if_changed(pending_download_format, None);
            return;
        }
        if (pending.is_some() || *pending_execute.read())
            && !*searched_once.read()
            && !*loading.read()
        {
            if let Some(fmt) = pending.as_deref() {
                log_info_evt(
                    "download",
                    "startup",
                    "auto_search_triggered",
                    Some(&format!("format={fmt}")),
                );
            } else {
                log_info_evt(
                    "search",
                    "startup",
                    "auto_search_triggered",
                    Some("execute=true"),
                );
            }
            start_search(criteria, locale, pending.is_some(), search_runtime);
            set_signal_if_changed(pending_execute, false);
        }
    });

    // ── Download dispatch: wait for SPARQL query, then stream the file ────
    use_effect(move || {
        let pending = pending_download_format.read().clone();
        if let Some(fmt) = pending {
            if *loading.read() {
                if !*waiting_loading_logged.peek() {
                    log_debug_evt(
                        "download",
                        "dispatch",
                        "waiting_loading",
                        Some(&format!("format={fmt}")),
                    );
                    set_signal_if_changed(waiting_loading_logged, true);
                }
                set_signal_if_changed(waiting_query_logged, false);
                return;
            }
            set_signal_if_changed(waiting_loading_logged, false);
            if let Some(query) = sparql_query.read().as_deref() {
                set_signal_if_changed(waiting_query_logged, false);
                let crit = criteria.peek().clone();
                match DownloadFormat::from_str(&fmt) {
                    Some(format) => {
                        let q = query.to_string();
                        let filename = export::generate_filename(&crit, format.extension());
                        set_signal_if_changed(pending_download_format, None);
                        set_signal_if_changed(download_dispatching, true);
                        log_debug_evt(
                            "download",
                            "startup_dispatch",
                            "query_check",
                            Some(&format!(
                                "format={} has_SERVICE={} has_SELECT={} query_bytes={}",
                                format.log_name(),
                                q.contains("SERVICE"),
                                q.contains("SELECT"),
                                q.len()
                            )),
                        );
                        spawn(async move {
                            log_info_evt(
                                "download",
                                "dispatch",
                                "started",
                                Some(&format!("format={}", format.log_name())),
                            );
                            if let Err(err) = execute_download(
                                format,
                                #[cfg(target_arch = "wasm32")]
                                Arc::new(crit.clone()),
                                q,
                                filename,
                            )
                            .await
                            {
                                log_warn_evt(
                                    "download",
                                    "dispatch",
                                    "error",
                                    Some(&format!("format={} reason={err}", format.log_name())),
                                );
                            }
                            set_signal_if_changed(download_dispatching, false);
                        });
                    }
                    None => {
                        log_warn_evt(
                            "download",
                            "dispatch",
                            "unsupported_format",
                            Some(&format!("format={fmt}")),
                        );
                        set_signal_if_changed(error_kind, ErrorKind::Validation);
                        set_signal_if_changed(
                            error,
                            Some(err_unsupported_format(*locale.peek(), &fmt)),
                        );
                        set_signal_if_changed(pending_download_format, None);
                        set_signal_if_changed(download_dispatching, false);
                    }
                }
            } else if !*waiting_query_logged.peek() {
                log_debug_evt(
                    "download",
                    "dispatch",
                    "waiting_query",
                    Some(&format!("format={fmt}")),
                );
                set_signal_if_changed(waiting_query_logged, true);
            }
        } else {
            set_signal_if_changed(waiting_loading_logged, false);
            set_signal_if_changed(waiting_query_logged, false);
        }
    });

    let on_search = move |_| start_search(criteria, locale, false, search_runtime);
    let on_preview = move |_| start_search(criteria, locale, false, search_runtime);

    rsx! {
        a { class: "skip-link", href: "#main-panel", "{t(locale_value, TextKey::SkipToResults)}" }
        div { class: if *app_view.read() == AppView::Explore { "app-layout" } else { "app-layout no-sidebar" },

            // ── Left sidebar (Explore tab only) ───────────────────────────
            if *app_view.read() == AppView::Explore {
                aside {
                    class: if mobile_open { "sidebar mobile-open" } else { "sidebar mobile-closed" },
                    aria_label: "{t(locale_value, TextKey::SearchFilters)}",
                    button {
                        class: "filters-toggle",
                        r#type: "button",
                        aria_label: if mobile_open { t(locale_value, TextKey::FiltersHide) } else { t(locale_value, TextKey::FiltersShow) },
                        aria_expanded: if mobile_open { "true" } else { "false" },
                        onclick: move |_| {
                            let next = !*mobile_filters_open.peek();
                            *mobile_filters_open.write() = next;
                        },
                        if mobile_open {
                            "{t(locale_value, TextKey::FiltersHide)}"
                        } else {
                            "{t(locale_value, TextKey::FiltersShow)}"
                        }
                    }
                    SearchPanel { on_search }
                    div { class: "sidebar-logo-wrap",
                        a {
                            href: "?",
                            title: "{t(locale_value, TextKey::PageTitle)}",
                            aria_label: "{t(locale_value, TextKey::PageTitle)}",
                            img {
                                class: "sidebar-logo",
                                src: "assets/lotus_ferris.svg",
                                alt: "LOTUS Ferris logo",
                                width: "180",
                                height: "180",
                                loading: "lazy",
                                decoding: "async",
                            }
                        }
                    }
                }
            }

            // ── Main panel ────────────────────────────────────────────────
            main {
                id: "main-panel",
                class: if *app_view.read() == AppView::Explore { "main-content" } else { "main-content single-pane" },
                tabindex: "-1",

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
                        // Language switcher
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
                    // View-tab switcher
                    nav {
                        class: "view-switch",
                        role: "group",
                        aria_label: "{view_switch_aria(locale_value)}",
                        button {
                            class: if *app_view.read() == AppView::Explore { "btn btn-xs lang-btn active" } else { "btn btn-xs lang-btn" },
                            r#type: "button",
                            aria_pressed: if *app_view.read() == AppView::Explore { "true" } else { "false" },
                            onclick: move |_| app_view.set(AppView::Explore),
                            "{view_label_explorer(locale_value)}"
                        }
                        button {
                            class: if *app_view.read() == AppView::Curation { "btn btn-xs lang-btn active" } else { "btn btn-xs lang-btn" },
                            r#type: "button",
                            aria_pressed: if *app_view.read() == AppView::Curation { "true" } else { "false" },
                            onclick: move |_| app_view.set(AppView::Curation),
                            "{view_label_curation_explorer(locale_value)}"
                        }
                        button {
                            class: if *app_view.read() == AppView::Draw { "btn btn-xs lang-btn active" } else { "btn btn-xs lang-btn" },
                            r#type: "button",
                            aria_pressed: if *app_view.read() == AppView::Draw { "true" } else { "false" },
                            onclick: move |_| app_view.set(AppView::Draw),
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

                // ── Tab content ───────────────────────────────────────────
                if *app_view.read() == AppView::Explore {
                    div { class: "page-header-meta",
                        HeaderMetaSection {
                            resolved_qid,
                            query_hash,
                            result_hash,
                            total_matches,
                            locale,
                        }
                    }
                    ShareNotice { shareable_url, locale }
                    TaxonNotice { taxon_notice, locale }
                    ErrorNotice {
                        error,
                        error_kind,
                        locale,
                        loading,
                        on_dismiss: move |_| *error.write() = None,
                        on_retry: move |_| start_search(criteria, locale, false, search_runtime),
                    }
                    ResultsViewport { on_preview }
                } else if *app_view.read() == AppView::Curation {
                    components::data_curation_page::DataCurationPage { locale: locale_value }
                } else {
                    DrawPage { locale: locale_value }
                }

                Footer { locale: locale_value }
            }
        }
    }
}

// ── Download format validation helper (used in tests only) ───────────────────

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
