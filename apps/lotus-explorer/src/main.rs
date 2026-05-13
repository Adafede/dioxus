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

use app::view::AppView;
use components::copy_button::CopyButton;
use components::data_curation_page::DataCurationPage;
use components::layout::footer::Footer;
use components::layout::header_meta::HeaderMetaSection;
use components::layout::notices::{ShareNotice, TaxonNotice};
use components::results_table::ResultsTable;
use components::search_panel::{KetcherPanel, SearchPanel};
use dioxus::prelude::*;
use download::{DownloadFormat, execute_download};
use features::explore::search_state::{
    SearchMetrics, SearchRuntime, emit_search_summary, set_signal_if_changed,
};
use features::explore::types::{AppError, ErrorKind, QueryPhase};
use features::explore::search_utils::{compute_hashes, sanitize_taxon_input};
use features::explore::url_state::{
    absolute_current_url_with_query, build_shareable_url, initial_criteria_from_url,
    initial_download_format_from_url, initial_execute_from_url, initial_locale_from_url,
    initial_view_from_url, persist_locale_query_param, persist_view_query_param,
};
#[cfg(target_arch = "wasm32")]
use i18n::error_hint_memory;
use i18n::{
    Locale, TextKey, err_invalid_search_input, err_query_stage_failed, err_taxon_not_found,
    err_taxon_parse_failed, err_taxon_resolution_failed, err_taxon_search_failed,
    err_unsupported_format, t, view_label_curation_explorer, view_label_draw, view_label_explorer,
    view_switch_aria, warn_ambiguous_taxon, warn_input_standardized,
};
use models::*;
use state::{ResultsContext, SearchUiContext, use_results_context};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};
use utils::logging::{log_debug_evt, log_info_evt, log_timing_evt, log_warn_evt};

type TaxonCache = BTreeMap<String, String>;

fn taxon_cache() -> &'static Mutex<TaxonCache> {
    static CACHE: OnceLock<Mutex<TaxonCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn cache_lookup_taxon_qid(name: &str) -> Option<String> {
    let key = name.trim().to_lowercase();
    if key.is_empty() {
        return None;
    }
    let guard = taxon_cache().lock().ok()?;
    guard.get(&key).cloned()
}

fn cache_store_taxon_qid(name: &str, qid: &str) {
    let key = name.trim().to_lowercase();
    if key.is_empty() || qid.trim().is_empty() {
        return;
    }
    if let Ok(mut guard) = taxon_cache().lock() {
        guard.insert(key, qid.to_string());
    }
}

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
    // These guard whether a particular waiting-state debug message has already
    // been emitted this cycle. They are never read in RSX; `.peek()` is used
    // for reads so they never subscribe effects/components to themselves.
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

    // Memoised derived state — recomputed only when their inputs change.
    // If we have precise totals from the parser, use them directly. Otherwise,
    // fall back to counting over the display slice.
    let shareable_url =
        use_memo(move || build_shareable_url(&criteria.read()).map(Arc::<str>::from));

    use_effect(move || {
        persist_locale_query_param(*locale.read());
    });

    use_effect(move || {
        persist_view_query_param(*app_view.read());
    });

    // ── Search handler ────────────────────────────────────────────────────────
    let on_search = move |_| start_search(criteria, locale, false, search_runtime);
    let on_preview = move |_| start_search(criteria, locale, false, search_runtime);

    // Programmatic flow: when URL contains `download=true&format=...`, run the
    // search automatically and trigger download once query materializes.
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
            } else {
                if !*waiting_query_logged.peek() {
                    log_debug_evt(
                        "download",
                        "dispatch",
                        "waiting_query",
                        Some(&format!("format={fmt}")),
                    );
                    set_signal_if_changed(waiting_query_logged, true);
                }
            }
        } else {
            set_signal_if_changed(waiting_loading_logged, false);
            set_signal_if_changed(waiting_query_logged, false);
        }
    });

    rsx! {

        a { class: "skip-link", href: "#main-panel", "{t(locale_value, TextKey::SkipToResults)}" }
        div { class: if *app_view.read() == AppView::Explore { "app-layout" } else { "app-layout no-sidebar" },
            // ── Left sidebar ──────────────────────────────────────────────
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
                    DataCurationPage { locale: locale_value }
                } else {
                    DrawPage { locale: locale_value }
                }

                Footer { locale: locale_value }
            }
        }
    }
}

#[component]
fn DrawPage(locale: Locale) -> Element {
    rsx! {
        section { class: "draw-wrap", aria_label: "{view_label_draw(locale)}",
            KetcherPanel { locale }
        }
    }
}

/// Top-level results area. Subscribes only to `loading`, `entries`,
/// `error` (presence check), and `searched_once` to decide which view to
/// show. `query_phase` is consumed inside `LoadingState` so phase
/// transitions don't trigger a full viewport re-render.
#[component]
fn ResultsViewport(on_preview: EventHandler<()>) -> Element {
    let state = use_results_context();
    let locale = *state.locale.read();
    let loading = *state.loading.read();
    let has_error = state.error.read().is_some();
    let searched_once = *state.searched_once.read();
    let download_only_mode = *state.download_only_mode.read();
    let download_dispatching = *state.download_dispatching.read();
    let entries = state.entries;

    if loading {
        return rsx! {
            LoadingState { locale }
        };
    }

    if entries.read().is_empty() && !has_error && !searched_once {
        return rsx! {
            WelcomeScreen { locale }
        };
    }

    if entries.read().is_empty() && !has_error && download_only_mode && download_dispatching {
        return rsx! {
            DownloadDispatchState { locale }
        };
    }

    if entries.read().is_empty() && !has_error && download_only_mode {
        return rsx! {
            DownloadOnlyState { locale, on_preview }
        };
    }

    rsx! {
        ResultsTable {}
    }
}

#[component]
fn DownloadDispatchState(locale: Locale) -> Element {
    rsx! {
        div {
            class: "loading-state",
            role: "status",
            aria_live: "polite",
            aria_busy: "true",
            div { class: "spinner-lg", "aria-hidden": "true" }
            p { "{t(locale, TextKey::PreparingDownload)}" }
            p { class: "loading-hint", "{t(locale, TextKey::WelcomeProgrammaticDownload)}" }
        }
    }
}

#[component]
fn DownloadOnlyState(locale: Locale, on_preview: EventHandler<()>) -> Element {
    rsx! {
        div { class: "notice notice-info", role: "status",
            span { class: "notice-label", "{t(locale, TextKey::Notice)}" }
            span { class: "notice-value", "{t(locale, TextKey::WelcomeProgrammaticDownload)}" }
            button {
                class: "btn btn-sm",
                r#type: "button",
                onclick: move |_| on_preview.call(()),
                "{t(locale, TextKey::RunSearch)}"
            }
        }
    }
}

/// Spinner overlay shown while a query is in-flight.
/// Subscribes to `query_phase` independently so phase-text updates
/// (ResolvingTaxon → Counting → FetchingPreview) don't propagate to
/// `ResultsViewport` or its siblings.
#[component]
fn LoadingState(locale: Locale) -> Element {
    let state = use_results_context();
    let query_phase = *state.query_phase.read();
    rsx! {
        div {
            class: "loading-state",
            role: "status",
            aria_live: "polite",
            aria_busy: "true",
            div { class: "spinner-lg", "aria-hidden": "true" }
            p { "{query_phase_text(locale, query_phase)}" }
            p { class: "loading-hint", "{t(locale, TextKey::LoadingHint)}" }
        }
    }
}

// ── Welcome screen ────────────────────────────────────────────────────────────

#[component]
fn WelcomeScreen(locale: Locale) -> Element {
    rsx! {
        section { class: "welcome",
            div { class: "welcome-hero",
                h2 { "{t(locale, TextKey::WelcomeTitle)}" }
                p { class: "welcome-lead",
                    "{t(locale, TextKey::WelcomeLeadA)}"
                    "{t(locale, TextKey::WelcomeLeadB)}"
                    a {
                        href: "https://www.wikidata.org/wiki/Q104225190",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "LOTUS initiative"
                    }
                    "{t(locale, TextKey::WelcomeLeadC)}"
                    a {
                        href: "https://www.wikidata.org/",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "Wikidata"
                    }
                    "{t(locale, TextKey::WelcomeLeadD)}"
                    a {
                        href: "https://qlever.dev/wikidata",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "QLever"
                    }
                    "{t(locale, TextKey::WelcomeLeadE)}"
                }
            }

            div { class: "welcome-examples",
                h3 { "{t(locale, TextKey::WelcomeTry)}" }
                ul { class: "example-list",
                    ExRow {
                        value: "taxon=<name|QID>",
                        note: t(locale, TextKey::ExampleGentiana),
                    }
                    ExRow {
                        value: "*",
                        note: t(locale, TextKey::ExampleAllTriples),
                    }
                    ExRow {
                        value: "structure=<SMILES|Molfile>",
                        note: t(locale, TextKey::ExampleSmilesOnly),
                    }
                }
                p { class: "form-hint welcome-cli-hint",
                    "{t(locale, TextKey::WelcomeProgrammaticDownload)}"
                }
                p { class: "form-hint", "{t(locale, TextKey::LabelLanguagePolicy)}" }
                div { class: "welcome-cli-list",
                    DownloadExampleRow {
                        locale,
                        format: t(locale, TextKey::ExampleQueryExecute),
                        query: "?taxon=Gentiana%20lutea&execute=true",
                    }
                    DownloadExampleRow {
                        locale,
                        format: t(locale, TextKey::ExampleQueryTaxon),
                        query: "?taxon=*&download=true&format=csv",
                    }
                    DownloadExampleRow {
                        locale,
                        format: t(locale, TextKey::ExampleQueryStructure),
                        query: "?structure=c1ccccc1&structure_search_type=similarity&smiles_threshold=0.85&download=true&format=json",
                    }
                    DownloadExampleRow {
                        locale,
                        format: t(locale, TextKey::ExampleQueryAdvanced),
                        query: "?taxon=Fungi&mass_filter=true&mass_min=0&mass_max=300&year_filter=true&year_start=2000&year_end=2026&formula_filter=true&c_min=1&c_max=10&cl_state=required&br_state=excluded&download=true&format=rdf",
                    }
                }
            }
        }
    }
}

#[component]
fn DownloadExampleRow(locale: Locale, format: &'static str, query: &'static str) -> Element {
    let absolute = absolute_current_url_with_query(query.trim_start_matches('?'));
    let absolute = Arc::<str>::from(absolute);
    rsx! {
        div { class: "welcome-cli-row",
            span { class: "welcome-cli-format mono", "{format}" }
            code { class: "mono welcome-cli-query", "{query}" }
            CopyButton { text: absolute, locale }
        }
    }
}

#[component]
fn ExRow(value: &'static str, note: &'static str) -> Element {
    rsx! {
        li { class: "example-item",
            code { class: "example-value", "{value}" }
            span { class: "example-note", "{note}" }
        }
    }
}

// ── Error notice ──────────────────────────────────────────────────────────────

#[component]
fn ErrorNotice(
    error: Signal<Option<String>>,
    error_kind: Signal<ErrorKind>,
    locale: Signal<Locale>,
    loading: Signal<bool>,
    on_dismiss: EventHandler<()>,
    on_retry: EventHandler<()>,
) -> Element {
    let locale = *locale.read();
    let kind = *error_kind.read();
    let is_loading = *loading.read();
    let err_ref = error.read();
    let Some(msg) = err_ref.as_deref() else {
        return rsx! {};
    };
    rsx! {
        div { class: "notice notice-error", role: "alert",
            span { class: "notice-label", "{t(locale, TextKey::Error)}" }
            span { class: "notice-value", "{msg}" }
            span { class: "notice-value", "{error_hint_text(locale, kind)}" }
            if is_retryable(kind) && !is_loading {
                button {
                    class: "btn btn-sm",
                    r#type: "button",
                    onclick: move |_| on_retry.call(()),
                    "{t(locale, TextKey::Retry)}"
                }
            }
            button {
                class: "notice-dismiss",
                r#type: "button",
                aria_label: "{t(locale, TextKey::DismissError)}",
                onclick: move |_| on_dismiss.call(()),
                "×"
            }
        }
    }
}

// ── Async search ─────────────────────

struct SearchOutcome {
    rows: Vec<CompoundEntry>,
    qid: Option<String>,
    warning: Option<String>,
    query: String,
    total_matches: Option<usize>,
    total_stats: Option<DatasetStats>,
    display_capped_rows: bool,
}

fn start_search(
    criteria: Signal<SearchCriteria>,
    locale: Signal<Locale>,
    direct_download_mode: bool,
    runtime: SearchRuntime,
) {
    let SearchRuntime {
        executed_criteria,
        loading,
        error,
        error_kind,
        query_phase,
        searched_once,
        download_only_mode,
        download_dispatching,
        mut entries,
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
        mut search_request_token,
    } = runtime;
    let crit = criteria.peek().clone();

    let request_token = {
        let mut next = search_request_token.write();
        *next += 1;
        *next
    };
    if *loading.peek() {
        log_info_evt(
            "search",
            "start",
            "superseding_inflight",
            Some(&format!("request_token={request_token}")),
        );
    }

    if !crit.is_valid() {
        log_warn_evt(
            "search",
            "start",
            "validation_failed",
            Some("reason=missing_taxon_and_structure"),
        );
        set_signal_if_changed(error, Some(err_invalid_search_input(*locale.peek())));
        set_signal_if_changed(error_kind, ErrorKind::Validation);
        return;
    }

    // Freeze the criteria snapshot that produced the current result lifecycle.
    set_signal_if_changed(executed_criteria, crit.clone());

    set_signal_if_changed(error, None);
    set_signal_if_changed(error_kind, ErrorKind::Unknown);
    set_signal_if_changed(searched_once, true);
    set_signal_if_changed(download_only_mode, direct_download_mode);
    set_signal_if_changed(download_dispatching, false);
    log_info_evt("search", "start", "loading_true", None);
    set_signal_if_changed(loading, true);
    log_debug_evt("search", "ResolvingTaxon", "entered", None);
    set_signal_if_changed(query_phase, QueryPhase::ResolvingTaxon);
    *entries.write() = Arc::<[CompoundEntry]>::from([]);
    set_signal_if_changed(taxon_notice, None);
    set_signal_if_changed(resolved_qid, None);
    set_signal_if_changed(query_hash, None);
    set_signal_if_changed(result_hash, None);
    set_signal_if_changed(sparql_query, None);
    set_signal_if_changed(metadata_json, None);
    set_signal_if_changed(total_matches, None);
    set_signal_if_changed(total_stats, None);
    set_signal_if_changed(display_capped_rows, false);
    set_signal_if_changed(mobile_filters_open, false);

    spawn(async move {
        match do_search(
            crit.clone(),
            *locale.peek(),
            query_phase,
            direct_download_mode,
        )
        .await
        {
            Ok(outcome) => {
                if request_token != *search_request_token.peek() {
                    log_debug_evt(
                        "search",
                        "finish",
                        "stale_result_ignored",
                        Some(&format!("request_token={request_token}")),
                    );
                    return;
                }
                let filtered_stats = if direct_download_mode {
                    None
                } else {
                    Some(
                        outcome
                            .total_stats
                            .clone()
                            .unwrap_or_else(|| DatasetStats::from_entries(&outcome.rows)),
                    )
                };
                let filtered_matches = if direct_download_mode {
                    None
                } else {
                    Some(outcome.total_matches.unwrap_or(outcome.rows.len()))
                };

                let (q_hash, r_hash) =
                    compute_hashes(outcome.qid.as_deref().unwrap_or(""), &crit, &outcome.rows);
                let meta_str = export::build_metadata_json(export::MetadataInputs {
                    criteria: &crit,
                    qid: outcome.qid.as_deref(),
                    number_of_records_override: filtered_matches,
                    query_hash: &q_hash,
                    result_hash: &r_hash,
                });

                let display_slice: Rows = Arc::from(outcome.rows.into_boxed_slice());
                log_debug_evt("search", "Rendering", "entered", None);
                set_signal_if_changed(query_phase, QueryPhase::Rendering);
                set_signal_if_changed(resolved_qid, outcome.qid);
                set_signal_if_changed(taxon_notice, outcome.warning);
                set_signal_if_changed(query_hash, Some(q_hash));
                set_signal_if_changed(result_hash, Some(r_hash));
                set_signal_if_changed(sparql_query, Some(Arc::<str>::from(outcome.query)));
                set_signal_if_changed(metadata_json, Some(Arc::<str>::from(meta_str)));
                set_signal_if_changed(display_capped_rows, outcome.display_capped_rows);
                set_signal_if_changed(total_matches, filtered_matches);
                set_signal_if_changed(total_stats, filtered_stats);
                *entries.write() = display_slice;
                log_info_evt("search", "finish", "loading_false", Some("result=success"));
                set_signal_if_changed(loading, false);
                log_debug_evt("search", "Idle", "entered", None);
                set_signal_if_changed(query_phase, QueryPhase::Idle);
            }
            Err(e) => {
                if request_token != *search_request_token.peek() {
                    log_debug_evt(
                        "search",
                        "finish",
                        "stale_error_ignored",
                        Some(&format!("request_token={request_token}")),
                    );
                    return;
                }
                set_signal_if_changed(error_kind, e.kind);
                set_signal_if_changed(error, Some(e.message));
                log_info_evt("search", "finish", "loading_false", Some("result=error"));
                set_signal_if_changed(loading, false);
                log_debug_evt("search", "Idle", "entered", None);
                set_signal_if_changed(query_phase, QueryPhase::Idle);
            }
        }
    });
}

async fn do_search(
    crit: SearchCriteria,
    locale: Locale,
    mut query_phase: Signal<QueryPhase>,
    direct_download_mode: bool,
) -> Result<SearchOutcome, AppError> {
    let search_timer = perf::start_timer("LOTUS:search_total");
    let mut metrics = SearchMetrics::default();
    log_info_evt("search", "start", "begin", None);
    let taxon = crit.taxon.trim().to_string();
    // Preserve Molfile blocks verbatim — leading blank lines and whitespace
    // on header rows (lines 1–3 of a V2000/V3000 CTAB) are significant and
    // must reach SACHEM untouched, otherwise the query silently returns
    // no matches. Only trim single-line SMILES inputs.
    let smiles = {
        let normalized = crit.smiles.replace("\r\n", "\n").replace('\r', "\n");
        let kind = queries::classify_structure(&normalized);
        if matches!(
            kind,
            queries::StructureKind::MolfileV2000 | queries::StructureKind::MolfileV3000
        ) {
            normalized
        } else {
            normalized.trim().to_string()
        }
    };

    if !direct_download_mode && let Some(api_base) = api::api_base_url() {
        log_info_evt(
            "search",
            "api",
            "path_selected",
            Some(&format!("base={}", api_base)),
        );
        let mut api_crit = crit.clone();
        api_crit.smiles = smiles.clone();
        let display_limit = runtime_table_row_limit();
        // Keep counts exact for both taxon-only and SACHEM searches so
        // displayed totals match full-file exports.
        let include_counts = true;
        log_info_evt(
            "search",
            "api",
            "attempt",
            Some(&format!(
                "base={} limit={} include_counts={}",
                api_base, display_limit, include_counts
            )),
        );
        let api_timer = perf::start_timer("LOTUS:api_search");
        match api::search(&api_crit, display_limit, include_counts).await {
            Ok(response) => {
                let api_elapsed = perf::end_timer("LOTUS:api_search", api_timer);
                metrics.add_network(api_elapsed);
                log_timing_evt(
                    "search",
                    "api",
                    "success",
                    api_elapsed,
                    Some(&format!(
                        "rows={} total_matches={}",
                        response.rows.len(),
                        response.total_matches
                    )),
                );
                let display_capped_rows = if include_counts {
                    response.total_matches > response.rows.len()
                } else {
                    response.rows.len() >= display_limit
                };
                let rows = response
                    .rows
                    .into_iter()
                    .map(CompoundEntry::from)
                    .collect::<Vec<_>>();

                return Ok(SearchOutcome {
                    rows,
                    qid: response.resolved_taxon_qid,
                    warning: response.warning,
                    // Keep the API-provided query as the canonical export/query-panel source.
                    query: response.query,
                    total_matches: Some(response.total_matches),
                    total_stats: Some(response.stats.into()),
                    display_capped_rows,
                });
            }
            Err(err) => {
                let api_elapsed = perf::end_timer("LOTUS:api_search", api_timer);
                log_timing_evt(
                    "search",
                    "api",
                    "fallback_direct",
                    api_elapsed,
                    Some(&format!("reason={err}")),
                );
            }
        }
    } else {
        let reason = if direct_download_mode {
            "reason=direct_download_mode"
        } else {
            "reason=api_not_configured"
        };
        log_info_evt("search", "api", "path_not_available", Some(reason));
    }

    let mut warning: Option<String> = None;
    let taxon_qid: Option<String> = if taxon.is_empty() {
        None
    } else if taxon == "*" {
        Some("*".to_string())
    } else if taxon.to_uppercase().starts_with('Q')
        && taxon[1..].chars().all(|c| c.is_ascii_digit())
    {
        Some(taxon.to_uppercase())
    } else {
        log_debug_evt("search", "ResolvingTaxon", "entered", None);
        *query_phase.write() = QueryPhase::ResolvingTaxon;
        let taxon_timer = perf::start_timer("LOTUS:taxon_resolution");
        log_debug_evt(
            "search",
            "ResolvingTaxon",
            "querying",
            Some(&format!("taxon_input={taxon}")),
        );
        let sanitized = sanitize_taxon_input(&taxon);
        if sanitized != taxon {
            warning = Some(warn_input_standardized(locale, &taxon, &sanitized));
        }

        if let Some(cached_qid) = cache_lookup_taxon_qid(&sanitized) {
            let taxon_elapsed = perf::end_timer("LOTUS:taxon_resolution", taxon_timer);
            log_timing_evt(
                "search",
                "ResolvingTaxon",
                "cache_hit",
                taxon_elapsed,
                Some(&format!("taxon_input={} qid={}", sanitized, cached_qid)),
            );
            Some(cached_qid)
        } else {
            let query = queries::query_taxon_search(&sanitized);
            let csv = sparql::execute_sparql_bytes(&query)
                .await
                .map_err(|e| AppError {
                    kind: ErrorKind::Network,
                    message: err_taxon_search_failed(locale, &e.to_string()),
                })?;
            let taxon_elapsed = perf::end_timer("LOTUS:taxon_resolution", taxon_timer);
            metrics.add_network(taxon_elapsed);
            perf::log_timing(
                "ResolvingTaxon",
                "Taxon query completed",
                Some(taxon_elapsed),
            );
            let matches = sparql::parse_taxon_csv_bytes(&csv).map_err(|e| AppError {
                kind: ErrorKind::Parse,
                message: err_taxon_parse_failed(locale, &e.to_string()),
            })?;
            if matches.is_empty() {
                return Err(AppError {
                    kind: ErrorKind::Validation,
                    message: err_taxon_not_found(locale, &taxon),
                });
            }
            let lower = sanitized.to_lowercase();
            let exact: Vec<&TaxonMatch> = matches
                .iter()
                .filter(|m| m.name.to_lowercase() == lower)
                .collect();
            let best = exact
                .first()
                .copied()
                .or_else(|| matches.first())
                .ok_or_else(|| AppError {
                    kind: ErrorKind::Parse,
                    message: err_taxon_resolution_failed(locale),
                })?;
            if exact.len() > 1 || (exact.is_empty() && matches.len() > 1) {
                let names = matches
                    .iter()
                    .take(4)
                    .map(|m| format!("{} ({})", m.name, m.qid))
                    .collect::<Vec<_>>()
                    .join(", ");
                warning = Some(warn_ambiguous_taxon(locale, &best.name, &best.qid, &names));
            }
            cache_store_taxon_qid(&sanitized, &best.qid);
            Some(best.qid.clone())
        }
    };

    let sparql_query = if !smiles.is_empty() {
        let effective_type = if (smiles.contains('\n') || smiles.contains('\r'))
            && crit.smiles_search_type == SmilesSearchType::Similarity
        {
            SmilesSearchType::Substructure
        } else {
            crit.smiles_search_type
        };
        let taxon_for_sachem = match taxon_qid.as_deref() {
            Some("*") => Some("Q2382443"),
            Some(qid) => Some(qid),
            None => None,
        };
        let q = queries::query_sachem(
            &smiles,
            effective_type,
            crit.smiles_threshold,
            taxon_for_sachem,
        );
        log_debug_evt(
            "search",
            "query_build",
            "sachem_query_created",
            Some(&format!("has_SERVICE={}", q.contains("SERVICE"))),
        );
        q
    } else {
        // Both None (no taxon) and Some("*") (wildcard) produce the all-compounds query;
        // only a specific QID narrows the result set.
        match taxon_qid.as_deref() {
            Some(qid) if qid != "*" => queries::query_compounds_by_taxon(qid),
            _ => queries::query_all_compounds(),
        }
    };

    let execution_query = queries::query_with_server_filters(&sparql_query, &crit);
    log_debug_evt(
        "search",
        "query_build",
        "after_server_filters",
        Some(&format!(
            "has_SERVICE={} has_FILTER={}",
            execution_query.contains("SERVICE"),
            execution_query.contains("FILTER")
        )),
    );
    log_debug_evt(
        "search",
        "build_query",
        "ready",
        Some(&format!("query_bytes={}", execution_query.len())),
    );

    if direct_download_mode {
        let total_elapsed = perf::end_timer("LOTUS:search_total", search_timer);
        log_timing_evt(
            "search",
            "direct_download",
            "ready",
            total_elapsed,
            Some("skipped=count_and_preview"),
        );
        emit_search_summary(total_elapsed, metrics);
        return Ok(SearchOutcome {
            rows: Vec::new(),
            qid: taxon_qid,
            warning,
            query: execution_query,
            total_matches: None,
            total_stats: None,
            display_capped_rows: false,
        });
    }

    // Use the same count+preview flow for structure and non-structure searches
    // so UI totals, query panel, and downloads stay consistent.

    let display_limit = runtime_table_row_limit();
    // Fast path: fetch exact aggregate counts with a tiny response, then fetch
    // only the display window. This keeps metadata totals exact while cutting
    // transfer size for large result sets.
    log_debug_evt("search", "Counting", "entered", None);
    *query_phase.write() = QueryPhase::Counting;
    let count_query = queries::query_counts_from_base(&execution_query);
    let display_query = queries::query_with_limit(&execution_query, display_limit);

    let (rows, total_stats_out, total_matches, display_capped_rows) = match async {
        #[cfg(target_arch = "wasm32")]
        {
            // Avoid keeping count and preview response bodies alive at once on wasm.
            log_debug_evt("search", "Counting", "sequential_fetch_wasm", None);

            let count_timer = perf::start_timer("LOTUS:count_query");
            let counts_csv = sparql::execute_sparql_bytes(&count_query)
                .await
                .map_err(|e| AppError {
                    kind: ErrorKind::Network,
                    message: err_query_stage_failed(locale, "count query", &e.to_string()),
                })?;
            let count_elapsed = perf::end_timer("LOTUS:count_query", count_timer);
            metrics.add_network(count_elapsed);
            perf::log_timing("Counting", "Count query completed", Some(count_elapsed));

            let count_parse_timer = perf::start_timer("LOTUS:count_parse");
            let full_stats = sparql::parse_counts_csv_bytes(&counts_csv).map_err(|e| AppError {
                kind: ErrorKind::Parse,
                message: err_query_stage_failed(locale, "count parse", &e.to_string()),
            })?;
            let count_parse_elapsed = perf::end_timer("LOTUS:count_parse", count_parse_timer);
            metrics.add_parse(count_parse_elapsed);
            perf::log_timing(
                "Counting",
                &format!(
                    "Count parse completed (entries={}, compounds={}, taxa={}, refs={})",
                    full_stats.n_entries,
                    full_stats.n_compounds,
                    full_stats.n_taxa,
                    full_stats.n_references
                ),
                Some(count_parse_elapsed),
            );

            log_debug_evt("search", "FetchingPreview", "entered", None);
            *query_phase.write() = QueryPhase::FetchingPreview;

            let display_timer = perf::start_timer("LOTUS:display_query");
            let display_csv = sparql::execute_sparql_bytes(&display_query)
                .await
                .map_err(|e| AppError {
                    kind: ErrorKind::Network,
                    message: err_query_stage_failed(locale, "display query", &e.to_string()),
                })?;
            let display_elapsed = perf::end_timer("LOTUS:display_query", display_timer);
            metrics.add_network(display_elapsed);
            perf::log_timing(
                "FetchingPreview",
                "Display query completed",
                Some(display_elapsed),
            );

            let display_parse_timer = perf::start_timer("LOTUS:display_parse");
            let rows = sparql::parse_compounds_csv_display_bytes(&display_csv, display_limit)
                .map_err(|e| AppError {
                    kind: ErrorKind::Parse,
                    message: err_query_stage_failed(locale, "display parse", &e.to_string()),
                })?;
            let display_parse_elapsed = perf::end_timer("LOTUS:display_parse", display_parse_timer);
            metrics.add_parse(display_parse_elapsed);
            perf::log_timing(
                "FetchingPreview",
                &format!("Display parse completed (rows={})", rows.len()),
                Some(display_parse_elapsed),
            );
            let display_capped_rows = full_stats.n_entries > rows.len();

            Ok::<_, AppError>((
                rows,
                Some(full_stats.clone()),
                Some(full_stats.n_entries),
                display_capped_rows,
            ))
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            log_debug_evt("search", "Counting", "parallel_fetch_started", None);
            let count_fetch = async {
                let count_timer = perf::start_timer("LOTUS:count_query");
                let counts_csv = sparql::execute_sparql_bytes(&count_query)
                    .await
                    .map_err(|e| AppError {
                        kind: ErrorKind::Network,
                        message: err_query_stage_failed(locale, "count query", &e.to_string()),
                    })?;
                let count_elapsed = perf::end_timer("LOTUS:count_query", count_timer);
                Ok::<_, AppError>((counts_csv, count_elapsed))
            };

            let display_fetch = async {
                let display_timer = perf::start_timer("LOTUS:display_query");
                let display_csv =
                    sparql::execute_sparql_bytes(&display_query)
                        .await
                        .map_err(|e| AppError {
                            kind: ErrorKind::Network,
                            message: err_query_stage_failed(
                                locale,
                                "display query",
                                &e.to_string(),
                            ),
                        })?;
                let display_elapsed = perf::end_timer("LOTUS:display_query", display_timer);
                Ok::<_, AppError>((display_csv, display_elapsed))
            };

            let ((counts_csv, count_elapsed), (display_csv, display_elapsed)) =
                futures::try_join!(count_fetch, display_fetch)?;

            metrics.add_network(count_elapsed);
            metrics.add_network(display_elapsed);

            perf::log_timing("Counting", "Count query completed", Some(count_elapsed));
            perf::log_timing(
                "FetchingPreview",
                "Display query completed",
                Some(display_elapsed),
            );

            let count_parse_timer = perf::start_timer("LOTUS:count_parse");
            let full_stats = sparql::parse_counts_csv_bytes(&counts_csv).map_err(|e| AppError {
                kind: ErrorKind::Parse,
                message: err_query_stage_failed(locale, "count parse", &e.to_string()),
            })?;
            let count_parse_elapsed = perf::end_timer("LOTUS:count_parse", count_parse_timer);
            metrics.add_parse(count_parse_elapsed);
            perf::log_timing(
                "Counting",
                &format!(
                    "Count parse completed (entries={}, compounds={}, taxa={}, refs={})",
                    full_stats.n_entries,
                    full_stats.n_compounds,
                    full_stats.n_taxa,
                    full_stats.n_references
                ),
                Some(count_parse_elapsed),
            );

            log_debug_evt("search", "FetchingPreview", "entered", None);
            *query_phase.write() = QueryPhase::FetchingPreview;

            let display_parse_timer = perf::start_timer("LOTUS:display_parse");
            let rows = sparql::parse_compounds_csv_display_bytes(&display_csv, display_limit)
                .map_err(|e| AppError {
                    kind: ErrorKind::Parse,
                    message: err_query_stage_failed(locale, "display parse", &e.to_string()),
                })?;
            let display_parse_elapsed = perf::end_timer("LOTUS:display_parse", display_parse_timer);
            metrics.add_parse(display_parse_elapsed);
            perf::log_timing(
                "FetchingPreview",
                &format!("Display parse completed (rows={})", rows.len()),
                Some(display_parse_elapsed),
            );
            let display_capped_rows = full_stats.n_entries > rows.len();

            Ok::<_, AppError>((
                rows,
                Some(full_stats.clone()),
                Some(full_stats.n_entries),
                display_capped_rows,
            ))
        }
    }
    .await
    {
        Ok(v) => v,
        Err(err_msg) => {
            #[cfg(target_arch = "wasm32")]
            {
                return Err(AppError {
                    kind: ErrorKind::Memory,
                    message: i18n::err_wasm_large_query_fallback(locale, &err_msg.message),
                });
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                log_warn_evt(
                    "search",
                    "Fallback",
                    "entered",
                    Some("reason=two_phase_failed"),
                );
                let _ = err_msg;
                let fallback_query_timer = perf::start_timer("LOTUS:fallback_query");
                let csv = sparql::execute_sparql_bytes(&execution_query)
                    .await
                    .map_err(|e| AppError {
                        kind: ErrorKind::Network,
                        message: err_query_stage_failed(locale, "query", &e.to_string()),
                    })?;
                let fallback_query_elapsed =
                    perf::end_timer("LOTUS:fallback_query", fallback_query_timer);
                metrics.add_network(fallback_query_elapsed);
                perf::log_timing(
                    "Fallback",
                    "Fallback query completed",
                    Some(fallback_query_elapsed),
                );

                let fallback_parse_timer = perf::start_timer("LOTUS:fallback_parse");
                let (rows, full_stats, parse_capped) =
                    sparql::parse_compounds_csv_capped_bytes(&csv, display_limit).map_err(|e| {
                        AppError {
                            kind: ErrorKind::Parse,
                            message: err_query_stage_failed(locale, "parse", &e.to_string()),
                        }
                    })?;
                let fallback_parse_elapsed =
                    perf::end_timer("LOTUS:fallback_parse", fallback_parse_timer);
                metrics.add_parse(fallback_parse_elapsed);
                perf::log_timing(
                    "Fallback",
                    &format!("Fallback parse completed (rows={})", rows.len()),
                    Some(fallback_parse_elapsed),
                );
                let display_capped_rows = parse_capped || full_stats.n_entries > rows.len();
                (
                    rows,
                    Some(full_stats.clone()),
                    Some(full_stats.n_entries),
                    display_capped_rows,
                )
            }
        }
    };

    let outcome = SearchOutcome {
        rows,
        qid: taxon_qid,
        warning,
        query: execution_query,
        total_matches,
        total_stats: total_stats_out,
        display_capped_rows,
    };
    let total_elapsed = perf::end_timer("LOTUS:search_total", search_timer);
    perf::log_timing(
        "SearchComplete",
        &format!(
            "Search completed (display_rows={}, total_matches={})",
            outcome.rows.len(),
            outcome.total_matches.unwrap_or(outcome.rows.len())
        ),
        Some(total_elapsed),
    );
    emit_search_summary(total_elapsed, metrics);
    Ok(outcome)
}


#[cfg(test)]
fn is_supported_download_format(fmt: &str) -> bool {
    DownloadFormat::from_str(fmt).is_some()
}

fn query_phase_text(locale: Locale, phase: QueryPhase) -> &'static str {
    match phase {
        QueryPhase::Idle => t(locale, TextKey::LoadingTitle),
        QueryPhase::ResolvingTaxon => t(locale, TextKey::LoadingResolvingTaxon),
        QueryPhase::Counting => t(locale, TextKey::LoadingCounting),
        QueryPhase::FetchingPreview => t(locale, TextKey::LoadingFetchingPreview),
        QueryPhase::Rendering => t(locale, TextKey::LoadingRendering),
    }
}

fn is_retryable(kind: ErrorKind) -> bool {
    matches!(
        kind,
        ErrorKind::Network | ErrorKind::Parse | ErrorKind::Unknown
    )
}

fn error_hint_text(locale: Locale, kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::Validation => t(locale, TextKey::ErrorHintValidation),
        ErrorKind::Network => t(locale, TextKey::ErrorHintNetwork),
        ErrorKind::Parse => t(locale, TextKey::ErrorHintParse),
        #[cfg(target_arch = "wasm32")]
        ErrorKind::Memory => error_hint_memory(locale),
        ErrorKind::Unknown => t(locale, TextKey::ErrorHintUnknown),
    }
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
