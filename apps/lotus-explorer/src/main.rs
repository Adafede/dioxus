#![allow(non_snake_case)]

mod components;
mod export;
mod i18n;
mod models;
mod queries;
mod sparql;

use components::copy_button::CopyButton;
use components::results_table::ResultsTable;
use components::search_panel::{KetcherPanel, SearchPanel};
use dioxus::prelude::*;
use i18n::{
    Locale, TextKey, err_invalid_search_input, err_taxon_not_found, t, warn_ambiguous_taxon,
    warn_input_standardized,
};
use models::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq)]
enum QueryPhase {
    Idle,
    ResolvingTaxon,
    Counting,
    FetchingPreview,
    Rendering,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ErrorKind {
    Validation,
    Network,
    Server,
    Parse,
    Memory,
    Unknown,
}

fn main() {
    console_log::init_with_level(log::Level::Debug).ok();
    launch(App);
}

// ── Root component ────────────────────────────────────────────────────────────

#[component]
fn App() -> Element {
    let criteria: Signal<SearchCriteria> = use_signal(initial_criteria_from_url);
    let locale: Signal<Locale> = use_signal(initial_locale_from_url);
    // Entries live behind an `Arc<[…]>` so prop/signal clones are a single
    // refcount bump instead of duplicating the whole result buffer.
    let entries: Signal<Rows> = use_signal(|| Arc::<[CompoundEntry]>::from([]));
    let loading: Signal<bool> = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let error_kind: Signal<ErrorKind> = use_signal(|| ErrorKind::Unknown);
    let query_phase: Signal<QueryPhase> = use_signal(|| QueryPhase::Idle);
    let searched_once: Signal<bool> = use_signal(|| false);
    let taxon_notice: Signal<Option<String>> = use_signal(|| None);
    let resolved_qid: Signal<Option<String>> = use_signal(|| None);
    let query_hash: Signal<Option<String>> = use_signal(|| None);
    let result_hash: Signal<Option<String>> = use_signal(|| None);
    let sparql_query: Signal<Option<String>> = use_signal(|| None);
    let metadata_json: Signal<Option<String>> = use_signal(|| None);
    let total_matches: Signal<Option<usize>> = use_signal(|| None);
    let total_stats: Signal<Option<DatasetStats>> = use_signal(|| None);
    let sort: Signal<SortState> = use_signal(SortState::default);
    let page: Signal<usize> = use_signal(|| 0usize);
    let mut mobile_filters_open: Signal<bool> = use_signal(|| false);

    // Memoised derived state — recomputed only when their inputs change.
    // If we have precise totals from the parser, use them directly. Otherwise,
    // fall back to counting over the display slice.
    let stats = use_memo(move || match total_stats.read().as_ref() {
        Some(s) => s.clone(),
        None => DatasetStats::from_entries(&entries.read()),
    });
    let shareable_url =
        use_memo(move || build_shareable_url(&criteria.read()).map(Arc::<str>::from));

    // ── Search handler ────────────────────────────────────────────────────────
    let on_search = move |_| {
        start_search(
            criteria,
            locale,
            loading,
            error,
            error_kind,
            query_phase,
            searched_once,
            entries,
            taxon_notice,
            resolved_qid,
            query_hash,
            result_hash,
            sparql_query,
            metadata_json,
            total_matches,
            total_stats,
            page,
            mobile_filters_open,
        )
    };

    rsx! {

        a { class: "skip-link", href: "#results-section", "{t(*locale.read(), TextKey::SkipToResults)}" }
        div { class: "app-layout",
            // ── Left sidebar ──────────────────────────────────────────────
            aside { class: if *mobile_filters_open.read() { "sidebar mobile-open" } else { "sidebar mobile-closed" },
                button {
                    class: "filters-toggle",
                    r#type: "button",
                    aria_label: if *mobile_filters_open.read() { t(*locale.read(), TextKey::FiltersHide) } else { t(*locale.read(), TextKey::FiltersShow) },
                    aria_expanded: if *mobile_filters_open.read() { "true" } else { "false" },
                    onclick: move |_| {
                        let next = !*mobile_filters_open.peek();
                        *mobile_filters_open.write() = next;
                    },
                    if *mobile_filters_open.read() {
                        "{t(*locale.read(), TextKey::FiltersHide)}"
                    } else {
                        "{t(*locale.read(), TextKey::FiltersShow)}"
                    }
                }
                SearchPanel {
                    criteria,
                    locale: *locale.read(),
                    on_search,
                    loading: *loading.read(),
                }
                div { class: "sidebar-logo-wrap",
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

            // ── Main panel ────────────────────────────────────────────────
            main { class: "main-content",
                div { class: "page-header",
                    div { class: "page-brand",
                        h1 { class: "page-title", "{t(*locale.read(), TextKey::PageTitle)}" }
                    }
                    p { class: "page-sub", "{t(*locale.read(), TextKey::PageSubtitle)}" }
                    if let Some(qid) = resolved_qid.read().as_deref() {
                        p { class: "page-meta",
                            span { class: "meta-key", "{t(*locale.read(), TextKey::ResolvedTaxon)}" }
                            span { class: "meta-sep", ":" }
                            span { class: "meta-val mono", "{qid}" }
                            CopyButton {
                                text: qid.to_string(),
                                title: t(*locale.read(), TextKey::CopyTaxonQid),
                                locale: *locale.read(),
                            }
                        }
                    }
                    if let (Some(qh), Some(rh)) = (
                        query_hash.read().as_deref(),
                        result_hash.read().as_deref(),
                    )
                    {
                        p { class: "page-meta",
                            span { class: "meta-key", "{t(*locale.read(), TextKey::QueryHash)}" }
                            span { class: "meta-sep", ":" }
                            span { class: "meta-val mono", "{&qh[..12]}" }
                            CopyButton {
                                text: qh.to_string(),
                                title: t(*locale.read(), TextKey::CopyFullQueryHash),
                                locale: *locale.read(),
                            }
                            span { class: "meta-sep", "·" }
                            span { class: "meta-key", "{t(*locale.read(), TextKey::ResultHash)}" }
                            span { class: "meta-sep", ":" }
                            span { class: "meta-val mono", "{&rh[..12]}" }
                            CopyButton {
                                text: rh.to_string(),
                                title: t(*locale.read(), TextKey::CopyFullResultHash),
                                locale: *locale.read(),
                            }
                        }
                    }
                    if let Some(n) = *total_matches.read() {
                        p { class: "page-meta",
                            span { class: "meta-key", "{t(*locale.read(), TextKey::TotalMatches)}" }
                            span { class: "meta-sep", ":" }
                            span { class: "meta-val mono", "{n}" }
                        }
                    }
                }

                KetcherPanel { locale: *locale.read() }

                if let Some(share) = shareable_url.read().as_deref() {
                    div { class: "notice notice-info", role: "status",
                        span { class: "notice-label", "{t(*locale.read(), TextKey::Share)}" }
                        input {
                            class: "notice-value notice-copy-field mono",
                            r#type: "text",
                            readonly: true,
                            value: "{share}",
                        }
                        CopyButton {
                            text: absolute_share_url(share),
                            title: t(*locale.read(), TextKey::CopyShareableLink),
                            locale: *locale.read(),
                        }
                    }
                }

                if let Some(warning) = taxon_notice.read().as_deref() {
                    div { class: "notice notice-warn", role: "status",
                        span { class: "notice-label", "{t(*locale.read(), TextKey::Notice)}" }
                        span { class: "notice-value", "{warning}" }
                    }
                }

                if let Some(msg) = error.read().as_deref() {
                    div { class: "notice notice-error", role: "alert",
                        span { class: "notice-label", "{t(*locale.read(), TextKey::Error)}" }
                        span { class: "notice-value", "{msg}" }
                        span { class: "notice-value", "{error_hint_text(*locale.read(), *error_kind.read())}" }
                        if is_retryable(*error_kind.read()) && !*loading.read() {
                            button {
                                class: "btn btn-sm",
                                r#type: "button",
                                onclick: move |_| {
                                    start_search(
                                        criteria,
                                        locale,
                                        loading,
                                        error,
                                        error_kind,
                                        query_phase,
                                        searched_once,
                                        entries,
                                        taxon_notice,
                                        resolved_qid,
                                        query_hash,
                                        result_hash,
                                        sparql_query,
                                        metadata_json,
                                        total_matches,
                                        total_stats,
                                        page,
                                        mobile_filters_open,
                                    )
                                },
                                "{t(*locale.read(), TextKey::Retry)}"
                            }
                        }
                        button {
                            class: "notice-dismiss",
                            r#type: "button",
                            aria_label: "{t(*locale.read(), TextKey::DismissError)}",
                            onclick: move |_| *error.write() = None,
                            "×"
                        }
                    }
                }

                if *loading.read() {
                    div {
                        class: "loading-state",
                        role: "status",
                        aria_live: "polite",
                        aria_busy: "true",
                        div { class: "spinner-lg", "aria-hidden": "true" }
                        p { "{query_phase_text(*locale.read(), *query_phase.read())}" }
                        p { class: "loading-hint", "{t(*locale.read(), TextKey::LoadingHint)}" }
                    }
                } else if entries.read().is_empty() && error.read().is_none() && !*searched_once.read() {
                    WelcomeScreen { locale: *locale.read() }
                } else {
                    ResultsTable {
                        entries,
                        locale: *locale.read(),
                        stats: stats.read().clone(),
                        total_stats: total_stats.read().clone(),
                        total_matches: *total_matches.read(),
                        sort,
                        page,
                        sparql_query: sparql_query.read().clone(),
                        metadata_json: metadata_json.read().clone(),
                        query_hash: query_hash.read().clone(),
                        result_hash: result_hash.read().clone(),
                        criteria,
                    }
                }

                Footer { locale: *locale.read() }
            }
        }
    }
}

// ── Footer (same links as the Python notebook, cleaner markup) ───────────────

#[component]
fn Footer(locale: Locale) -> Element {
    rsx! {
        footer { class: "app-footer",
            FooterRow {
                label: t(locale, TextKey::FooterData),
                class: "footer-link data",
                links: &[
                    ("https://www.wikidata.org/wiki/Q104225190", "LOTUS Initiative"),
                    ("https://www.wikidata.org/", "Wikidata"),
                ],
            }
            FooterRow {
                label: t(locale, TextKey::FooterCode),
                class: "footer-link code",
                links: &[
                    (
                        "https://github.com/Adafede/dioxus/tree/main/apps/lotus-explorer",
                        "lotus-explorer",
                    ),
                ],
            }
            FooterRow {
                label: t(locale, TextKey::FooterTools),
                class: "footer-link tool",
                links: &[
                    ("https://github.com/cdk/depict", "CDK Depict"),
                    ("https://idsm.elixir-czech.cz/", "IDSM"),
                    ("https://doi.org/10.1186/s13321-018-0282-y", "Sachem"),
                    ("https://qlever.dev/wikidata", "QLever"),
                ],
            }
            div { class: "footer-row",
                span { class: "footer-label", "{t(locale, TextKey::FooterLicense)}" }
                a {
                    class: "footer-link muted",
                    href: "https://creativecommons.org/publicdomain/zero/1.0/",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "CC0 1.0"
                }
                span { class: "footer-aside", "{t(locale, TextKey::FooterForData)}" }
                span { class: "footer-sep", "·" }
                a {
                    class: "footer-link muted",
                    href: "https://www.gnu.org/licenses/agpl-3.0.html",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "AGPL-3.0"
                }
                span { class: "footer-aside", "{t(locale, TextKey::FooterForCode)}" }
            }
        }
    }
}

#[component]
fn FooterRow(
    label: &'static str,
    class: &'static str,
    links: &'static [(&'static str, &'static str)],
) -> Element {
    rsx! {
        div { class: "footer-row",
            span { class: "footer-label", "{label}" }
            for (i, (href, text)) in links.iter().enumerate() {
                if i > 0 {
                    span { class: "footer-sep", "·" }
                }
                a {
                    class: "{class}",
                    href: "{href}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "{text}"
                }
            }
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
                        value: "Gentiana lutea",
                        note: t(locale, TextKey::ExampleGentiana),
                    }
                    ExRow {
                        value: "Cannabis sativa",
                        note: t(locale, TextKey::ExampleCannabis),
                    }
                    ExRow {
                        value: "Q134630",
                        note: t(locale, TextKey::ExampleCitrusQid),
                    }
                    ExRow {
                        value: "*",
                        note: t(locale, TextKey::ExampleAllTriples),
                    }
                    ExRow {
                        value: "c1ccccc1",
                        note: t(locale, TextKey::ExampleSmilesOnly),
                    }
                }
            }
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

// ── Async search — mirrors Python LOTUSExplorer.search() ─────────────────────

struct SearchOutcome {
    rows: Vec<CompoundEntry>,
    qid: Option<String>,
    warning: Option<String>,
    query: String,
    total_matches: Option<usize>,
    total_stats: Option<DatasetStats>,
}

#[allow(clippy::too_many_arguments)]
fn start_search(
    criteria: Signal<SearchCriteria>,
    locale: Signal<Locale>,
    mut loading: Signal<bool>,
    mut error: Signal<Option<String>>,
    mut error_kind: Signal<ErrorKind>,
    mut query_phase: Signal<QueryPhase>,
    mut searched_once: Signal<bool>,
    mut entries: Signal<Rows>,
    mut taxon_notice: Signal<Option<String>>,
    mut resolved_qid: Signal<Option<String>>,
    mut query_hash: Signal<Option<String>>,
    mut result_hash: Signal<Option<String>>,
    mut sparql_query: Signal<Option<String>>,
    mut metadata_json: Signal<Option<String>>,
    mut total_matches: Signal<Option<usize>>,
    mut total_stats: Signal<Option<DatasetStats>>,
    mut page: Signal<usize>,
    mut mobile_filters_open: Signal<bool>,
) {
    if *loading.peek() {
        return;
    }
    let crit = criteria.peek().clone();

    if !crit.is_valid() {
        *error.write() = Some(err_invalid_search_input(*locale.peek()));
        *error_kind.write() = ErrorKind::Validation;
        return;
    }

    *error.write() = None;
    *error_kind.write() = ErrorKind::Unknown;
    *searched_once.write() = true;
    *loading.write() = true;
    *query_phase.write() = QueryPhase::ResolvingTaxon;
    *entries.write() = Arc::<[CompoundEntry]>::from([]);
    *taxon_notice.write() = None;
    *resolved_qid.write() = None;
    *query_hash.write() = None;
    *result_hash.write() = None;
    *sparql_query.write() = None;
    *metadata_json.write() = None;
    *total_matches.write() = None;
    *total_stats.write() = None;
    *page.write() = 0;
    *mobile_filters_open.write() = false;

    spawn(async move {
        match do_search(crit.clone(), *locale.peek(), query_phase).await {
            Ok(mut outcome) => {
                // Client-side post-filters (mass, year, formula).
                if crit.has_mass_filter() {
                    outcome.rows.retain(|e| {
                        e.mass
                            .map_or(false, |m| m >= crit.mass_min && m <= crit.mass_max)
                    });
                }
                if crit.has_year_filter() {
                    outcome.rows.retain(|e| {
                        e.pub_year
                            .map_or(true, |y| y >= crit.year_min && y <= crit.year_max)
                    });
                }
                if crit.has_formula_filter() {
                    outcome
                        .rows
                        .retain(|e| formula_matches(e.formula.as_deref(), &crit));
                }

                let (q_hash, r_hash) =
                    compute_hashes(outcome.qid.as_deref().unwrap_or(""), &crit, &outcome.rows);
                let full_stats = outcome
                    .total_stats
                    .clone()
                    .unwrap_or_else(|| DatasetStats::from_entries(&outcome.rows));
                let meta_str = export::build_metadata_json(export::MetadataInputs {
                    criteria: &crit,
                    qid: outcome.qid.as_deref(),
                    stats: &full_stats,
                    number_of_records_override: outcome.total_matches,
                    query_hash: &q_hash,
                    result_hash: &r_hash,
                });

                let display_slice: Rows = Arc::from(outcome.rows.into_boxed_slice());
                *query_phase.write() = QueryPhase::Rendering;
                *resolved_qid.write() = outcome.qid;
                *taxon_notice.write() = outcome.warning;
                *query_hash.write() = Some(q_hash);
                *result_hash.write() = Some(r_hash);
                *sparql_query.write() = Some(outcome.query);
                *metadata_json.write() = Some(meta_str);
                *total_matches.write() = outcome.total_matches;
                *total_stats.write() = outcome.total_stats;
                *entries.write() = display_slice;
                *loading.write() = false;
                *query_phase.write() = QueryPhase::Idle;
            }
            Err(e) => {
                *error_kind.write() = classify_error_kind(&e);
                *error.write() = Some(e);
                *loading.write() = false;
                *query_phase.write() = QueryPhase::Idle;
            }
        }
    });
}

async fn do_search(
    crit: SearchCriteria,
    locale: Locale,
    mut query_phase: Signal<QueryPhase>,
) -> Result<SearchOutcome, String> {
    let taxon = crit.taxon.trim().to_string();
    // Preserve Molfile blocks verbatim — leading blank lines and whitespace
    // on header rows (lines 1–3 of a V2000/V3000 CTAB) are significant and
    // must reach SACHEM untouched, otherwise the query silently returns
    // no matches. Only trim single-line SMILES inputs. Mirrors the Python
    // `validate_and_escape` behavior.
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
        *query_phase.write() = QueryPhase::ResolvingTaxon;
        let sanitized = sanitize_taxon_input(&taxon);
        let query = queries::query_taxon_search(&sanitized);
        let csv = sparql::execute_sparql(&query)
            .await
            .map_err(|e| format!("Taxon search failed: {e}"))?;
        let matches =
            sparql::parse_taxon_csv(&csv).map_err(|e| format!("Taxon parse failed: {e}"))?;
        if matches.is_empty() {
            return Err(err_taxon_not_found(locale, &taxon));
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
            .ok_or_else(|| "Taxon resolution failed".to_string())?;
        if sanitized != taxon {
            warning = Some(warn_input_standardized(locale, &taxon, &sanitized));
        }
        if exact.len() > 1 || (exact.is_empty() && matches.len() > 1) {
            let names = matches
                .iter()
                .take(4)
                .map(|m| format!("{} ({})", m.name, m.qid))
                .collect::<Vec<_>>()
                .join(", ");
            warning = Some(warn_ambiguous_taxon(locale, &best.name, &best.qid, &names));
        }
        Some(best.qid.clone())
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
        queries::query_sachem(
            &smiles,
            effective_type,
            crit.smiles_threshold,
            taxon_for_sachem,
        )
    } else {
        match taxon_qid.as_deref() {
            Some("*") => queries::query_all_compounds(),
            None => queries::query_all_compounds(),
            Some(qid) => queries::query_compounds_by_taxon(qid),
        }
    };

    // Fast path: fetch exact aggregate counts with a tiny response, then fetch
    // only the display window. This keeps metadata totals exact while cutting
    // transfer size for large result sets.
    *query_phase.write() = QueryPhase::Counting;
    let display_limit = runtime_table_row_limit();
    let count_query = queries::query_counts_from_base(&sparql_query);
    let display_query = queries::query_with_limit(&sparql_query, display_limit);

    let (rows, total_stats_out, total_matches) = match async {
        let counts_csv = sparql::execute_sparql(&count_query)
            .await
            .map_err(|e| format!("Count query failed: {e}"))?;
        let full_stats = sparql::parse_counts_csv(&counts_csv)
            .map_err(|e| format!("Count parse failed: {e}"))?;

        *query_phase.write() = QueryPhase::FetchingPreview;
        let display_csv = sparql::execute_sparql(&display_query)
            .await
            .map_err(|e| format!("Display query failed: {e}"))?;
        let rows = sparql::parse_compounds_csv_display(&display_csv, display_limit)
            .map_err(|e| format!("Display parse failed: {e}"))?;

        Ok::<_, String>((rows, Some(full_stats.clone()), Some(full_stats.n_entries)))
    }
    .await
    {
        Ok(v) => v,
        Err(err_msg) => {
            #[cfg(target_arch = "wasm32")]
            {
                return Err(i18n::err_wasm_large_query_fallback(locale, &err_msg));
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = err_msg;
                let csv = sparql::execute_sparql(&sparql_query)
                    .await
                    .map_err(|e| format!("Query failed: {e}"))?;
                let (rows, full_stats, _parse_capped) =
                    sparql::parse_compounds_csv_capped(&csv, display_limit)
                        .map_err(|e| format!("Parse error: {e}"))?;
                (rows, Some(full_stats.clone()), Some(full_stats.n_entries))
            }
        }
    };

    Ok(SearchOutcome {
        rows,
        qid: taxon_qid,
        warning,
        query: sparql_query,
        total_matches,
        total_stats: total_stats_out,
    })
}

fn sanitize_taxon_input(taxon: &str) -> String {
    // Mirrors Python `str.capitalize()` on the genus: upper-case the first
    // character, lower-case the rest of that first word. Leaves subsequent
    // words (species epithets, authors, etc.) untouched.
    let replaced = taxon.replace('_', " ");
    let parts: Vec<&str> = replaced.split_whitespace().collect();
    if parts.len() > 1 {
        let first = parts[0];
        if first.is_empty() {
            return replaced;
        }
        let mut first_cap = String::with_capacity(first.len());
        let mut chars = first.chars();
        if let Some(c) = chars.next() {
            for uc in c.to_uppercase() {
                first_cap.push(uc);
            }
        }
        for c in chars {
            for lc in c.to_lowercase() {
                first_cap.push(lc);
            }
        }
        let mut out = first_cap;
        out.push(' ');
        out.push_str(&parts[1..].join(" "));
        out
    } else {
        replaced
    }
}

fn formula_matches(formula: Option<&str>, crit: &SearchCriteria) -> bool {
    // Python semantics: rows with no formula are *not* filtered out
    // (match_filters returns True when formula is empty). Only active
    // sub-filters reject a row.
    let raw_formula = match formula {
        Some(f) if !f.trim().is_empty() => f,
        _ => return true,
    };
    let normalized = normalize_formula(raw_formula);
    let exact = crit.formula_exact.trim();
    if !exact.is_empty() {
        return normalized == normalize_formula(exact);
    }

    let parsed = parse_formula_counts(&normalized);
    for (elem, min, max, default_max) in crit.element_ranges() {
        // Skip inactive ranges — matches Python `ElementRange.is_active`.
        if min == 0 && max >= default_max {
            continue;
        }
        let n = *parsed.get(elem).unwrap_or(&0);
        if n < min || n > max {
            return false;
        }
    }

    element_state_matches(parsed.get("F").copied().unwrap_or(0), crit.f_state)
        && element_state_matches(parsed.get("Cl").copied().unwrap_or(0), crit.cl_state)
        && element_state_matches(parsed.get("Br").copied().unwrap_or(0), crit.br_state)
        && element_state_matches(parsed.get("I").copied().unwrap_or(0), crit.i_state)
}

/// Translate Unicode subscripts (`₀…₉`) to ASCII digits so the formula
/// parser matches Wikidata strings such as `C₁₅H₁₀O₅`.
fn normalize_formula(formula: &str) -> String {
    formula
        .chars()
        .map(|c| match c {
            '₀' => '0',
            '₁' => '1',
            '₂' => '2',
            '₃' => '3',
            '₄' => '4',
            '₅' => '5',
            '₆' => '6',
            '₇' => '7',
            '₈' => '8',
            '₉' => '9',
            _ => c,
        })
        .collect()
}

fn element_state_matches(count: i32, state: ElementState) -> bool {
    match state {
        ElementState::Allowed => true,
        ElementState::Required => count > 0,
        ElementState::Excluded => count == 0,
    }
}

fn parse_formula_counts(formula: &str) -> BTreeMap<String, i32> {
    let mut out = BTreeMap::new();
    let chars: Vec<char> = formula.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if !chars[i].is_ascii_uppercase() {
            i += 1;
            continue;
        }
        let mut symbol = String::new();
        symbol.push(chars[i]);
        i += 1;
        if i < chars.len() && chars[i].is_ascii_lowercase() {
            symbol.push(chars[i]);
            i += 1;
        }
        let start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        let count = if start < i {
            formula[start..i].parse::<i32>().unwrap_or(1)
        } else {
            1
        };
        *out.entry(symbol).or_insert(0) += count;
    }
    out
}

fn compute_hashes(
    qid: &str,
    criteria: &SearchCriteria,
    rows: &[CompoundEntry],
) -> (String, String) {
    let normalized_qid = if qid.trim().is_empty() { "*" } else { qid };
    let normalized_taxon = criteria.taxon.trim();
    let mut query_source = format!("{}|{}", normalized_qid, normalized_taxon);
    let params = criteria.shareable_query_params();
    if !params.is_empty() {
        query_source.push('|');
        query_source.push_str(
            &params
                .into_iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&"),
        );
    }
    let query_hash = to_hex_lower(&Sha256::digest(query_source.as_bytes()));

    let mut compounds = rows
        .iter()
        .map(|e| e.compound_qid.as_str())
        .collect::<Vec<_>>();
    compounds.sort_unstable();
    compounds.dedup();
    let result_source = compounds.join("|");
    let result_hash = to_hex_lower(&Sha256::digest(result_source.as_bytes()));

    (query_hash, result_hash)
}

fn to_hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn build_shareable_url(criteria: &SearchCriteria) -> Option<String> {
    let params = criteria.shareable_query_params();
    if params.is_empty() {
        return None;
    }
    let query = params
        .into_iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(&k), urlencoding::encode(&v)))
        .collect::<Vec<_>>()
        .join("&");
    Some(format!("?{query}"))
}

/// Turn a relative `?foo=bar` share fragment into an absolute URL rooted at
/// the current page — the form users actually want when they paste the link
/// into a chat / email. On native (no `window`) it just returns `share`.
fn absolute_share_url(share: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            let loc = win.location();
            if let (Ok(origin), Ok(pathname)) = (loc.origin(), loc.pathname()) {
                return format!("{origin}{pathname}{share}");
            }
        }
    }
    share.to_string()
}

fn initial_criteria_from_url() -> SearchCriteria {
    let mut criteria = SearchCriteria::default();
    let params = read_url_query_params();
    let has_explicit_taxon = params.get("taxon").is_some();
    let mut has_structure = false;

    if let Some(taxon) = params.get("taxon") {
        criteria.taxon = taxon.clone();
    }
    if let Some(structure) = params
        .get("structure")
        .cloned()
        .or_else(|| params.get("smiles").cloned())
    {
        criteria.smiles = structure;
        has_structure = true;
    }
    if let Some(search_type) = params
        .get("structure_search_type")
        .cloned()
        .or_else(|| params.get("smiles_search_type").cloned())
    {
        criteria.smiles_search_type = if search_type == "similarity" {
            SmilesSearchType::Similarity
        } else {
            SmilesSearchType::Substructure
        };
    }
    if let Some(threshold) = params.get("smiles_threshold") {
        if let Ok(v) = threshold.parse::<f64>() {
            criteria.smiles_threshold = v.clamp(0.05, 1.0);
        }
    }

    // Share links with only `?structure=...` should not inherit the default
    // taxon from `SearchCriteria::default()` (Gentiana lutea), otherwise the
    // pasted URL does not reproduce the sender's result set.
    if has_structure && !has_explicit_taxon {
        criteria.taxon.clear();
    }

    criteria
}

fn initial_locale_from_url() -> Locale {
    let params = read_url_query_params();
    let lang = params.get("lang").map(|v| v.as_str()).unwrap_or("");
    Locale::detect(lang)
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

fn classify_error_kind(msg: &str) -> ErrorKind {
    let m = msg.to_ascii_lowercase();
    if m.contains("please enter") || m.contains("veuillez") || m.contains("not found") {
        ErrorKind::Validation
    } else if m.contains("network") || m.contains("cors") || m.contains("timeout") {
        ErrorKind::Network
    } else if m.contains("http") || m.contains("gateway") || m.contains("server") {
        ErrorKind::Server
    } else if m.contains("parse") {
        ErrorKind::Parse
    } else if m.contains("memory") || m.contains("wasm") {
        ErrorKind::Memory
    } else {
        ErrorKind::Unknown
    }
}

fn is_retryable(kind: ErrorKind) -> bool {
    matches!(
        kind,
        ErrorKind::Network | ErrorKind::Server | ErrorKind::Parse | ErrorKind::Unknown
    )
}

fn error_hint_text(locale: Locale, kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::Validation => t(locale, TextKey::ErrorHintValidation),
        ErrorKind::Network => t(locale, TextKey::ErrorHintNetwork),
        ErrorKind::Server => t(locale, TextKey::ErrorHintServer),
        ErrorKind::Parse => t(locale, TextKey::ErrorHintParse),
        ErrorKind::Memory => t(locale, TextKey::ErrorHintMemory),
        ErrorKind::Unknown => t(locale, TextKey::ErrorHintUnknown),
    }
}

fn read_url_query_params() -> BTreeMap<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let mut out = BTreeMap::new();
        let Some(window) = web_sys::window() else {
            return out;
        };
        let Ok(search) = window.location().search() else {
            return out;
        };
        let query = search.trim_start_matches('?');
        for pair in query.split('&') {
            if pair.is_empty() {
                continue;
            }
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or_default();
            let val = parts.next().unwrap_or_default();
            let key_decoded = urlencoding::decode(key)
                .map(|v| v.into_owned())
                .unwrap_or_else(|_| key.to_string());
            let val_decoded = urlencoding::decode(val)
                .map(|v| v.into_owned())
                .unwrap_or_else(|_| val.to_string());
            out.insert(key_decoded, val_decoded);
        }
        out
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        BTreeMap::new()
    }
}
