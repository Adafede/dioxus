// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::download::{DownloadFormat, execute_download, trigger_download};
use crate::export;
use crate::features::explore::actions::ExploreAction;
use crate::features::explore::search_state::dispatch_explore_action;
use crate::i18n::{CountNoun, Locale, TextKey, count_label, t};
use crate::models::*;
use crate::perf;
use crate::state::use_results_context;
use dioxus::prelude::*;
use std::sync::Arc;

#[path = "results_table/row_cells.rs"]
mod row_cells;
use row_cells::{row_text, visible_rows_view};

#[path = "results_table/sort_helpers.rs"]
mod sort_helpers;
use sort_helpers::{aria_sort_for, sort_icon_for};

const TABLE_SCROLL_ID: &str = "results-table-scroll";
const VIRTUAL_OVERSCAN_ROWS: usize = 12;
const ROW_HEIGHT_PX_COMFORTABLE: usize = 114;
const TABLE_VIEWPORT_FALLBACK_PX: usize = 640;

// Download specs/constants shared by query and metadata actions.
#[derive(Clone, Copy)]
struct DownloadQuerySpec {
    format: DownloadFormat,
    status_key: TextKey,
    title_key: TextKey,
    label_key: TextKey,
}

#[derive(Clone, Copy)]
struct DownloadMetadataSpec {
    title_key: TextKey,
    label_key: TextKey,
}

const DOWNLOAD_QUERY_CSV_SPEC: DownloadQuerySpec = DownloadQuerySpec {
    format: DownloadFormat::Csv,
    status_key: TextKey::StartingCsvDownload,
    title_key: TextKey::DownloadCsvTitle,
    label_key: TextKey::DownloadCsvLabel,
};

const DOWNLOAD_QUERY_JSON_SPEC: DownloadQuerySpec = DownloadQuerySpec {
    format: DownloadFormat::Json,
    status_key: TextKey::PreparingJsonDownload,
    title_key: TextKey::DownloadJsonTitle,
    label_key: TextKey::DownloadJsonLabel,
};

const DOWNLOAD_QUERY_RDF_SPEC: DownloadQuerySpec = DownloadQuerySpec {
    format: DownloadFormat::Rdf,
    status_key: TextKey::PreparingRdfDownload,
    title_key: TextKey::DownloadRdfTitle,
    label_key: TextKey::DownloadRdfLabel,
};

const DOWNLOAD_METADATA_SPEC: DownloadMetadataSpec = DownloadMetadataSpec {
    title_key: TextKey::DownloadMetadataTitle,
    label_key: TextKey::DownloadMetadataLabel,
};

/// Human-facing QLever UI endpoint (for the "Open in QLever" deep-link).
const QLEVER_UI: &str = "https://qlever.dev/wikidata";
const DOWNLOAD_METADATA_MIME: &str = "application/ld+json";

fn log_download_evt(phase: &str, state: &str, details: Option<&str>) {
    let msg = match details {
        Some(d) if !d.is_empty() => format!("event=download phase={phase} state={state} {d}"),
        _ => format!("event=download phase={phase} state={state}"),
    };
    perf::log_info(&msg);
}

fn log_download_timing(
    phase: &str,
    state: &str,
    elapsed: std::time::Duration,
    details: Option<&str>,
) {
    let ms = elapsed.as_secs_f64() * 1000.0;
    let msg = match details {
        Some(d) if !d.is_empty() => {
            format!("event=download phase={phase} state={state} elapsed_ms={ms:.1} {d}")
        }
        _ => format!("event=download phase={phase} state={state} elapsed_ms={ms:.1}"),
    };
    perf::log_info(&msg);
}

fn spawn_query_download(
    format: DownloadFormat,
    status_message: String,
    _criteria_snapshot: SearchCriteria,
    filename: String,
    query: Arc<str>,
    mut download_busy: Signal<bool>,
    mut download_status: Signal<Option<String>>,
) {
    *download_busy.write() = true;
    *download_status.write() = Some(status_message);
    spawn(async move {
        log_download_evt(
            "table_dispatch",
            "started",
            Some(&format!("format={}", format.log_name())),
        );
        log_download_evt(
            "table_query",
            "check",
            Some(&format!(
                "format={} has_SERVICE={} query_bytes={}",
                format.log_name(),
                query.contains("SERVICE"),
                query.len()
            )),
        );
        if let Err(err) = execute_download(
            format,
            #[cfg(target_arch = "wasm32")]
            Arc::new(_criteria_snapshot),
            query.to_string(),
            filename,
        )
        .await
        {
            log_download_evt(
                "table_fetch",
                "error",
                Some(&format!("format={} reason={err}", format.log_name())),
            );
        }
        *download_busy.write() = false;
        *download_status.write() = None;
    });
}

fn dispatch_query_download_spec(
    spec: DownloadQuerySpec,
    locale: Locale,
    criteria_snapshot: SearchCriteria,
    filename: String,
    query: Arc<str>,
    download_busy: Signal<bool>,
    download_status: Signal<Option<String>>,
) {
    spawn_query_download(
        spec.format,
        t(locale, spec.status_key).to_string(),
        criteria_snapshot,
        filename,
        query,
        download_busy,
        download_status,
    );
}

fn dispatch_metadata_download_blob(filename: &str, body: &str) {
    log_download_evt("table_dispatch", "started", Some("format=metadata"));
    let trigger_timer = perf::start_timer("LOTUS:table_download_meta_trigger");
    trigger_download(filename, DOWNLOAD_METADATA_MIME, body);
    let trigger_elapsed = perf::end_timer("LOTUS:table_download_meta_trigger", trigger_timer);
    log_download_timing(
        "table_trigger",
        "success",
        trigger_elapsed,
        Some("format=metadata"),
    );
}

/// Renders the full results section.
///
/// Reactive surface is deliberately narrow: this component subscribes only to
/// `entries` (for the empty-state check and sort index) and `locale`.  All
/// query-panel / stats / download signals are delegated to `ResultsToolbar`,
/// which subscribes to them independently.  Sort interactions therefore only
/// re-render `VirtualizedResultsTable`, not the toolbar or stats bar.
#[component]
pub fn ResultsTable() -> Element {
    let state = use_results_context();
    let explore = state.explore;
    let locale = *state.locale.read();
    let entries_len = explore.read().entries.len();

    // Memoised sort: compute a permutation of row indices instead of cloning
    // the whole Vec to sort it. Recomputes only when `entries` or `sort`
    // actually change.
    let sorted_indices: Memo<Arc<[u32]>> = use_memo(move || {
        let snapshot = explore.read();
        let rows = snapshot.entries.clone();
        let s = snapshot.sort;
        let mut idx: Vec<u32> = (0..rows.len() as u32).collect();
        idx.sort_by(|&a, &b| {
            let ea = &rows[a as usize];
            let eb = &rows[b as usize];
            let cmp = match s.col {
                SortColumn::Name => ea.name.cmp(&eb.name),
                SortColumn::Mass => ea
                    .mass
                    .partial_cmp(&eb.mass)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Formula => ea.formula.cmp(&eb.formula),
                SortColumn::TaxonName => ea.taxon_name.cmp(&eb.taxon_name),
                SortColumn::PubYear => ea.pub_year.cmp(&eb.pub_year),
                SortColumn::RefTitle => ea.ref_title.cmp(&eb.ref_title),
            };
            if s.dir == SortDir::Desc {
                cmp.reverse()
            } else {
                cmp
            }
        });
        Arc::from(idx.into_boxed_slice())
    });

    let total = entries_len;

    rsx! {
        div { id: "results-section", class: "results-wrap",
            // Subscribes to query/stats/download signals; isolated from sort.
            ResultsToolbar { locale }

            if total == 0 {
                div { class: "empty-state",
                    p { "{t(locale, TextKey::NoResults)}" }
                }
            } else {
                VirtualizedResultsTable { explore, locale, sorted_indices }
            }
        }
    }
}

/// Toolbar: query panel + stats bar + download actions + capped-rows notice.
///
/// Reads sparql_query, metadata_json, query_hash, result_hash, executed_criteria,
/// total_stats, total_matches, display_capped_rows, and entries (for fallback
/// stats) from context.  Intentionally separate from `ResultsTable` so that
/// sort changes never cause toolbar re-renders.
#[component]
fn ResultsToolbar(locale: Locale) -> Element {
    let state = use_results_context();
    let explore = state.explore.read().clone();
    let entries = explore.entries.clone();
    let sparql_query = explore.sparql_query.clone();
    let metadata_json = explore.metadata_json.clone();
    let query_hash = explore.query_hash.clone();
    let result_hash = explore.result_hash.clone();
    let criteria = explore.executed_criteria.clone();
    let total_stats = explore.total_stats.clone();
    let total_matches = explore.total_matches;
    let display_capped_rows = explore.display_capped_rows;

    // Fallback stats are memoised so they don't rerun on unrelated re-renders.
    let fallback_stats: Memo<DatasetStats> = use_memo(move || DatasetStats::from_entries(&entries));
    let display_stats = total_stats
        .as_ref()
        .cloned()
        .unwrap_or_else(|| fallback_stats.read().clone());
    let stats_partial = false;
    let entries_value = total_matches.unwrap_or(display_stats.n_entries);
    let entries_unique_value = display_stats.n_entries_unique;

    rsx! {
        if let Some(q) = sparql_query.as_ref() {
            details { class: "query-panel",
                summary { "{t(locale, TextKey::SparqlQuery)}" }
                div { class: "query-panel-actions",
                    crate::components::copy_button::CopyButton {
                        text: q.clone(),
                        title: t(locale, TextKey::CopySparqlQuery),
                        locale,
                    }
                }
                pre { class: "query-text", "{q.as_ref()}" }
            }
        }
        // ── Stats + toolbar ───────────────────────────────────────────
        div { class: "results-toolbar",
            div {
                class: "stat-bar",
                role: "group",
                aria_label: "{t(locale, TextKey::DatasetStatistics)}",
                StatBadge {
                    locale,
                    value: display_stats.n_compounds,
                    secondary_value: None,
                    secondary_label: None,
                    noun: CountNoun::Compound,
                    plus: stats_partial,
                }
                StatBadge {
                    locale,
                    value: display_stats.n_taxa,
                    secondary_value: None,
                    secondary_label: None,
                    noun: CountNoun::Taxon,
                    plus: stats_partial,
                }
                StatBadge {
                    locale,
                    value: display_stats.n_references,
                    secondary_value: None,
                    secondary_label: None,
                    noun: CountNoun::Reference,
                    plus: stats_partial,
                }
                StatBadge {
                    locale,
                    value: entries_value,
                    secondary_value: (entries_unique_value != entries_value)
                                                                                                                                                                                                                                                    .then_some(entries_unique_value),
                    secondary_label: Some(t(locale, TextKey::Unique)),
                    noun: CountNoun::Entry,
                    plus: false,
                }
            }
            ResultsDownloadActions {
                locale,
                criteria: criteria.clone(),
                sparql_query: sparql_query.clone(),
                metadata_json: metadata_json.clone(),
                query_hash,
                result_hash,
            }
        }
        if display_capped_rows {
            div { class: "notice notice-warn", role: "status",
                span { class: "notice-label", "{t(locale, TextKey::Notice)}" }
                span { class: "notice-value", "{t(locale, TextKey::DisplayCappedHint)}" }
            }
        }
    }
}

#[component]
fn ResultsDownloadActions(
    locale: Locale,
    criteria: SearchCriteria,
    sparql_query: Option<Arc<str>>,
    metadata_json: Option<Arc<str>>,
    query_hash: Option<String>,
    result_hash: Option<String>,
) -> Element {
    let export_available = sparql_query.is_some() || metadata_json.is_some();
    let csv_filename = export::generate_filename(&criteria, "csv");
    let json_filename = export::generate_filename(&criteria, "json");
    let rdf_filename = export::generate_filename(&criteria, "rdf");
    let metadata_filename = match (query_hash.as_deref(), result_hash.as_deref()) {
        (Some(q), Some(r)) => format!("{q}_{r}_metadata.json"),
        _ => export::generate_filename(&criteria, "metadata.json"),
    };
    let qlever_ui_url = sparql_query
        .as_ref()
        .map(|q| format!("{QLEVER_UI}?query={}", urlencoding::encode(q.as_ref())));
    let download_busy = use_signal(|| false);
    let download_status: Signal<Option<String>> = use_signal(|| None);
    let download_status_text = download_status
        .read()
        .clone()
        .unwrap_or_else(|| t(locale, TextKey::PreparingDownload).to_string());

    rsx! {
        div { class: "toolbar-actions",
            if *download_busy.read() {
                span {
                    class: "btn btn-sm",
                    role: "status",
                    aria_live: "polite",
                    span { class: "spinner-sm", "aria-hidden": "true" }
                    {download_status_text}
                }
            }
            if export_available {
                div {
                    class: "dl-group",
                    role: "group",
                    aria_label: "{t(locale, TextKey::DownloadResults)}",
                    if let Some(query) = sparql_query.as_ref() {
                        button {
                            class: "btn btn-sm",
                            r#type: "button",
                            disabled: *download_busy.read(),
                            onclick: {
                                let q = query.clone();
                                let criteria_snapshot = criteria.clone();
                                let filename = csv_filename.clone();
                                move |_| {
                                    dispatch_query_download_spec(
                                        DOWNLOAD_QUERY_CSV_SPEC,
                                        locale,
                                        criteria_snapshot.clone(),
                                        filename.clone(),
                                        q.clone(),
                                        download_busy,
                                        download_status,
                                    );
                                }
                            },
                            title: "{t(locale, DOWNLOAD_QUERY_CSV_SPEC.title_key)}",
                            aria_label: "{t(locale, DOWNLOAD_QUERY_CSV_SPEC.title_key)}",
                            "{t(locale, DOWNLOAD_QUERY_CSV_SPEC.label_key)}"
                        }
                        button {
                            class: "btn btn-sm",
                            r#type: "button",
                            disabled: *download_busy.read(),
                            onclick: {
                                let q = query.clone();
                                let criteria_snapshot = criteria.clone();
                                let filename = json_filename.clone();
                                move |_| {
                                    dispatch_query_download_spec(
                                        DOWNLOAD_QUERY_JSON_SPEC,
                                        locale,
                                        criteria_snapshot.clone(),
                                        filename.clone(),
                                        q.clone(),
                                        download_busy,
                                        download_status,
                                    );
                                }
                            },
                            title: "{t(locale, DOWNLOAD_QUERY_JSON_SPEC.title_key)}",
                            aria_label: "{t(locale, DOWNLOAD_QUERY_JSON_SPEC.title_key)}",
                            "{t(locale, DOWNLOAD_QUERY_JSON_SPEC.label_key)}"
                        }
                        button {
                            class: "btn btn-sm",
                            r#type: "button",
                            disabled: *download_busy.read(),
                            onclick: {
                                let q = query.clone();
                                let criteria_snapshot = criteria.clone();
                                let filename = rdf_filename.clone();
                                move |_| {
                                    dispatch_query_download_spec(
                                        DOWNLOAD_QUERY_RDF_SPEC,
                                        locale,
                                        criteria_snapshot.clone(),
                                        filename.clone(),
                                        q.clone(),
                                        download_busy,
                                        download_status,
                                    );
                                }
                            },
                            title: "{t(locale, DOWNLOAD_QUERY_RDF_SPEC.title_key)}",
                            aria_label: "{t(locale, DOWNLOAD_QUERY_RDF_SPEC.title_key)}",
                            "{t(locale, DOWNLOAD_QUERY_RDF_SPEC.label_key)}"
                        }
                    }
                    if let Some(body) = metadata_json.as_ref() {
                        button {
                            class: "btn btn-sm",
                            r#type: "button",
                            disabled: *download_busy.read(),
                            onclick: {
                                let body = body.clone();
                                let filename = metadata_filename.clone();
                                move |_| {
                                    dispatch_metadata_download_blob(&filename, body.as_ref());
                                }
                            },
                            title: "{t(locale, DOWNLOAD_METADATA_SPEC.title_key)}",
                            aria_label: "{t(locale, DOWNLOAD_METADATA_SPEC.title_key)}",
                            "{t(locale, DOWNLOAD_METADATA_SPEC.label_key)}"
                        }
                    }
                }
            }
            if let Some(url) = qlever_ui_url.as_deref() {
                a {
                    class: "btn btn-sm",
                    href: "{url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{t(locale, TextKey::OpenInQleverTitle)}",
                    "{t(locale, TextKey::OpenInQlever)}"
                }
            }
        }
    }
}

#[component]
fn VirtualizedResultsTable(
    explore: Signal<crate::features::explore::search_state::ExploreState>,
    locale: Locale,
    sorted_indices: Memo<Arc<[u32]>>,
) -> Element {
    #[cfg(target_arch = "wasm32")]
    let first_visible_row = use_signal(|| 0usize);
    #[cfg(not(target_arch = "wasm32"))]
    let first_visible_row = use_signal(|| 0usize);

    #[cfg(target_arch = "wasm32")]
    let viewport_height_px = use_signal(|| TABLE_VIEWPORT_FALLBACK_PX);
    #[cfg(not(target_arch = "wasm32"))]
    let viewport_height_px = use_signal(|| TABLE_VIEWPORT_FALLBACK_PX);

    #[cfg(target_arch = "wasm32")]
    let mut scroll_host = use_signal(|| None::<web_sys::HtmlElement>);

    #[cfg(target_arch = "wasm32")]
    let mut scroll_raf_scheduled = use_signal(|| false);
    #[cfg(target_arch = "wasm32")]
    let mut scroll_raf_cb = use_signal(|| None::<wasm_bindgen::closure::Closure<dyn FnMut(f64)>>);
    #[cfg(target_arch = "wasm32")]
    let mut scroll_raf_id = use_signal(|| None::<i32>);

    #[cfg(target_arch = "wasm32")]
    use_drop(move || {
        if let Some(id) = *scroll_raf_id.peek() {
            if let Some(win) = web_sys::window() {
                let _ = win.cancel_animation_frame(id);
            }
        }
        *scroll_raf_id.write() = None;
        *scroll_raf_scheduled.write() = false;
        *scroll_raf_cb.write() = None;
    });

    let total = explore.read().entries.len();
    let row_height_px = ROW_HEIGHT_PX_COMFORTABLE;
    let current_sort = explore.read().sort;
    let window_rows = (((*viewport_height_px.read()).saturating_add(row_height_px - 1))
        / row_height_px)
        .max(1)
        .saturating_add(VIRTUAL_OVERSCAN_ROWS * 2);
    let first_row = (*first_visible_row.read()).min(total);
    let start_row = first_row.saturating_sub(VIRTUAL_OVERSCAN_ROWS);
    let end_row = start_row.saturating_add(window_rows).min(total);
    let top_spacer_px = start_row.saturating_mul(row_height_px);
    let bottom_spacer_px = total.saturating_sub(end_row).saturating_mul(row_height_px);
    let visible_count = end_row.saturating_sub(start_row);
    let row_text = row_text(locale);

    let toggle_sort = move |col: SortColumn| {
        move |_: Event<MouseData>| {
            dispatch_explore_action(explore, ExploreAction::SortToggled(col));
        }
    };

    rsx! {
        div {
            id: TABLE_SCROLL_ID,
            class: "table-scroll",
            onscroll: move |_| {
                #[cfg(target_arch = "wasm32")]
                {
                    use wasm_bindgen::JsCast;
                    let div = if let Some(existing) = scroll_host.peek().as_ref() {
                        existing.clone()
                    } else {
                        let Some(win) = web_sys::window() else {
                            return;
                        };
                        let Some(document) = win.document() else {
                            return;
                        };
                        let Some(node) = document.get_element_by_id(TABLE_SCROLL_ID) else {
                            return;
                        };
                        let Ok(found) = node.dyn_into::<web_sys::HtmlElement>() else {
                            return;
                        };
                        *scroll_host.write() = Some(found.clone());
                        found
                    };

                    // Coalesce multiple native scroll events into one update per frame.
                    if *scroll_raf_scheduled.peek() {
                        return;
                    }
                    *scroll_raf_scheduled.write() = true;

                    let mut first_visible_row_sig = first_visible_row;
                    let mut viewport_height_px_sig = viewport_height_px;
                    let mut scroll_raf_scheduled_sig = scroll_raf_scheduled;
                    let mut scroll_raf_cb_sig = scroll_raf_cb;
                    let mut scroll_raf_id_sig = scroll_raf_id;
                    let div_for_raf = div.clone();
                    let raf_cb = wasm_bindgen::closure::Closure::wrap(
                        Box::new(move |_ts: f64| {
                            let top = div_for_raf.scroll_top().max(0) as usize;
                            let height = div_for_raf.client_height().max(0) as usize;
                            let next_first = (top / row_height_px).min(total);
                            if next_first != *first_visible_row_sig.peek() {
                                *first_visible_row_sig.write() = next_first;
                            }
                            if height > 0 && height != *viewport_height_px_sig.peek() {
                                *viewport_height_px_sig.write() = height;
                            }
                            *scroll_raf_id_sig.write() = None;
                            *scroll_raf_scheduled_sig.write() = false;
                            *scroll_raf_cb_sig.write() = None;
                        }) as Box<dyn FnMut(f64)>,
                    );
                    *scroll_raf_cb.write() = Some(raf_cb);
                    let scheduled_id = if let Some(win) = web_sys::window() {
                        if let Some(cb) = scroll_raf_cb.peek().as_ref() {
                            win.request_animation_frame(cb.as_ref().unchecked_ref()).ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let Some(id) = scheduled_id {
                        *scroll_raf_id.write() = Some(id);
                    } else {
                        *scroll_raf_id.write() = None;
                        *scroll_raf_scheduled.write() = false;
                        *scroll_raf_cb.write() = None;
                    }
                }
            },
            table {
                class: "results-table",
                aria_label: "{t(locale, TextKey::TableTriplesAria)}",
                caption { class: "sr-only", "{t(locale, TextKey::TableTriplesAria)}" }
                thead {
                    tr {
                        th { class: "th-static", scope: "col", "{t(locale, TextKey::Structure)}" }
                        th {
                            class: "sort-th",
                            scope: "col",
                            aria_sort: "{aria_sort_for(&current_sort, SortColumn::Name)}",
                            button {
                                class: "sort-btn",
                                r#type: "button",
                                aria_label: "{t(locale, TextKey::Compound)}",
                                onclick: toggle_sort(SortColumn::Name),
                                "{t(locale, TextKey::Compound)} "
                                span { class: "sort-icon", "aria-hidden": "true",
                                    {sort_icon_for(&current_sort, SortColumn::Name)}
                                }
                            }
                        }
                        th {
                            class: "sort-th",
                            scope: "col",
                            aria_sort: "{aria_sort_for(&current_sort, SortColumn::Mass)}",
                            button {
                                class: "sort-btn",
                                r#type: "button",
                                aria_label: "{t(locale, TextKey::Mass)}",
                                onclick: toggle_sort(SortColumn::Mass),
                                "{t(locale, TextKey::Mass)} "
                                span { class: "sort-icon", "aria-hidden": "true",
                                    {sort_icon_for(&current_sort, SortColumn::Mass)}
                                }
                            }
                        }
                        th {
                            class: "sort-th",
                            scope: "col",
                            aria_sort: "{aria_sort_for(&current_sort, SortColumn::Formula)}",
                            button {
                                class: "sort-btn",
                                r#type: "button",
                                aria_label: "{t(locale, TextKey::Formula)}",
                                onclick: toggle_sort(SortColumn::Formula),
                                "{t(locale, TextKey::Formula)} "
                                span { class: "sort-icon", "aria-hidden": "true",
                                    {sort_icon_for(&current_sort, SortColumn::Formula)}
                                }
                            }
                        }
                        th {
                            class: "sort-th",
                            scope: "col",
                            aria_sort: "{aria_sort_for(&current_sort, SortColumn::TaxonName)}",
                            button {
                                class: "sort-btn",
                                r#type: "button",
                                aria_label: "{t(locale, TextKey::TaxonCol)}",
                                onclick: toggle_sort(SortColumn::TaxonName),
                                "{t(locale, TextKey::TaxonCol)} "
                                span { class: "sort-icon", "aria-hidden": "true",
                                    {sort_icon_for(&current_sort, SortColumn::TaxonName)}
                                }
                            }
                        }
                        th {
                            class: "sort-th",
                            scope: "col",
                            aria_sort: "{aria_sort_for(&current_sort, SortColumn::RefTitle)}",
                            button {
                                class: "sort-btn",
                                r#type: "button",
                                aria_label: "{t(locale, TextKey::Reference)}",
                                onclick: toggle_sort(SortColumn::RefTitle),
                                "{t(locale, TextKey::Reference)} "
                                span { class: "sort-icon", "aria-hidden": "true",
                                    {sort_icon_for(&current_sort, SortColumn::RefTitle)}
                                }
                            }
                        }
                        th {
                            class: "sort-th",
                            scope: "col",
                            aria_sort: "{aria_sort_for(&current_sort, SortColumn::PubYear)}",
                            button {
                                class: "sort-btn",
                                r#type: "button",
                                aria_label: "{t(locale, TextKey::Year)}",
                                onclick: toggle_sort(SortColumn::PubYear),
                                "{t(locale, TextKey::Year)} "
                                span { class: "sort-icon", "aria-hidden": "true",
                                    {sort_icon_for(&current_sort, SortColumn::PubYear)}
                                }
                            }
                        }
                    }
                }
                tbody {
                    if top_spacer_px > 0 {
                        tr { class: "virtual-spacer-row", aria_hidden: "true",
                            td {
                                class: "virtual-spacer-cell",
                                colspan: "7",
                                style: "height: {top_spacer_px}px;",
                            }
                        }
                    }
                    {
                        // Keep a single read for each reactive source per window render.
                        let rows = explore.read().entries.clone();
                        let order = sorted_indices.read();
                        {
                            visible_rows_view(
                                locale,
                                row_text,
                                &rows,
                                order.as_ref(),
                                start_row,
                                visible_count,
                            )
                        }
                    }
                    if bottom_spacer_px > 0 {
                        tr { class: "virtual-spacer-row", aria_hidden: "true",
                            td {
                                class: "virtual-spacer-cell",
                                colspan: "7",
                                style: "height: {bottom_spacer_px}px;",
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Sub-components ────────────────────────────────────────────────────────────

#[component]
fn StatBadge(
    locale: Locale,
    value: usize,
    secondary_value: Option<usize>,
    secondary_label: Option<&'static str>,
    noun: CountNoun,
    plus: bool,
) -> Element {
    let display_value = if plus {
        format!("{value}+")
    } else {
        value.to_string()
    };
    let label = count_label(locale, noun, value);
    rsx! {
        div { class: "stat-badge",
            div { class: "stat-value-row",
                span { class: "stat-value", "{display_value}" }
                if let Some(secondary) = secondary_value {
                    div { class: "stat-secondary-row",
                        span { class: "stat-value-secondary mono", "{secondary}" }
                        if let Some(label) = secondary_label {
                            span { class: "stat-secondary-label", "{label}" }
                        }
                    }
                }
            }
            span { class: "stat-label", "{label}" }
        }
    }
}
