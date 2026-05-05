// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::download::{DownloadFormat, execute_download, trigger_download};
use crate::export;
use crate::i18n::{
    CountNoun, Locale, TextKey, aria_chemical_structure, aria_search_inchikey,
    aria_wikidata_entity, aria_wikidata_statement, count_label, t,
};
use crate::models::*;
use crate::perf;
use crate::state::use_results_context;
use dioxus::prelude::*;
use std::sync::Arc;

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
    criteria_snapshot: SearchCriteria,
    filename: String,
    query: String,
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
        if let Err(err) =
            execute_download(format, Arc::new(criteria_snapshot), query, filename).await
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
    query: String,
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

#[component]
pub fn ResultsTable() -> Element {
    let state = use_results_context();
    let entries = state.entries;
    let total_stats_signal = state.total_stats;
    let locale = *state.locale.read();
    let total_stats = total_stats_signal.read().clone();
    let total_matches = *state.total_matches.read();
    let display_capped_rows = *state.display_capped_rows.read();
    let sort = state.sort;
    let page = state.page;
    let sparql_query = state.sparql_query.read().clone();
    let metadata_json = state.metadata_json.read().clone();
    let query_hash = state.query_hash.read().clone();
    let result_hash = state.result_hash.read().clone();
    let criteria = state.executed_criteria;
    // Keep expensive fallback stats O(n) work memoized so UI-only signals
    // (like download spinner state) don't recompute from all rows.
    let fallback_stats: Memo<DatasetStats> =
        use_memo(move || DatasetStats::from_entries(&entries.read()));
    // Exports are served from fresh endpoint requests (CSV/JSON/RDF), so
    // the table only keeps the preview rows needed for rendering.
    let display_stats = total_stats
        .as_ref()
        .cloned()
        .unwrap_or_else(|| fallback_stats.read().clone());
    let total = entries.read().len();
    let stats_partial = false;
    let entries_value = total_matches.unwrap_or(display_stats.n_entries);
    let entries_unique_value = display_stats.n_entries_unique;

    // Memoised sort: compute a permutation of row indices instead of cloning
    // the whole Vec to sort it. Recomputes only when `entries` or `sort`
    // actually change.
    let sorted_indices: Memo<Arc<[u32]>> = use_memo(move || {
        let rows = entries.read();
        let s = *sort.read();
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

    // ── Export filenames & URLs (memoised; only rebuild when inputs change) ──
    let export_available = sparql_query.is_some() || metadata_json.is_some();

    let csv_filename = use_memo(move || {
        let c = criteria.read();
        export::generate_filename(&c, "csv")
    });
    let json_filename = use_memo(move || {
        let c = criteria.read();
        export::generate_filename(&c, "json")
    });
    let rdf_filename = use_memo(move || {
        let c = criteria.read();
        export::generate_filename(&c, "rdf")
    });

    // Metadata filename mirrors: `{query_hash}_{result_hash}_metadata.json`.
    let metadata_filename = match (query_hash.as_deref(), result_hash.as_deref()) {
        (Some(q), Some(r)) => format!("{q}_{r}_metadata.json"),
        _ => {
            let c = criteria.read();
            export::generate_filename(&c, "metadata.json")
        }
    };

    let qlever_ui_url = sparql_query
        .as_deref()
        .map(|q| format!("{QLEVER_UI}?query={}", urlencoding::encode(q)));
    let download_busy = use_signal(|| false);
    let download_status: Signal<Option<String>> = use_signal(|| None);
    let download_status_text = download_status
        .read()
        .clone()
        .unwrap_or_else(|| t(locale, TextKey::PreparingDownload).to_string());

    rsx! {
        div { id: "results-section", class: "results-wrap",
            if let Some(q) = sparql_query.as_deref() {
                details { class: "query-panel",
                    summary { "{t(locale, TextKey::SparqlQuery)}" }
                    div { class: "query-panel-actions",
                        crate::components::copy_button::CopyButton {
                            text: q.to_string(),
                            title: t(locale, TextKey::CopySparqlQuery),
                            locale,
                        }
                    }
                    pre { class: "query-text", "{q}" }
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
                            if let Some(query) = sparql_query.as_deref() {
                                button {
                                    class: "btn btn-sm",
                                    r#type: "button",
                                    disabled: *download_busy.read(),
                                    onclick: {
                                        let q = query.to_string();
                                        move |_| {
                                            dispatch_query_download_spec(
                                                DOWNLOAD_QUERY_CSV_SPEC,
                                                locale,
                                                criteria.read().clone(),
                                                csv_filename.read().clone(),
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
                                        let q = query.to_string();
                                        move |_| {
                                            dispatch_query_download_spec(
                                                DOWNLOAD_QUERY_JSON_SPEC,
                                                locale,
                                                criteria.read().clone(),
                                                json_filename.read().clone(),
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
                                        let q = query.to_string();
                                        move |_| {
                                            dispatch_query_download_spec(
                                                DOWNLOAD_QUERY_RDF_SPEC,
                                                locale,
                                                criteria.read().clone(),
                                                rdf_filename.read().clone(),
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
                                            dispatch_metadata_download_blob(&filename, &body);
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

            if display_capped_rows {
                div { class: "notice notice-warn", role: "status",
                    span { class: "notice-label", "{t(locale, TextKey::Notice)}" }
                    span { class: "notice-value", "{t(locale, TextKey::DisplayCappedHint)}" }
                }
            }

            if total == 0 {
                div { class: "empty-state",
                    p { "{t(locale, TextKey::NoResults)}" }
                }
            } else {
                VirtualizedResultsTable {
                    entries,
                    locale,
                    sort,
                    page,
                    sorted_indices,
                }
            }
        }
    }
}

#[component]
fn VirtualizedResultsTable(
    entries: ReadSignal<Rows>,
    locale: Locale,
    sort: Signal<SortState>,
    page: Signal<usize>,
    sorted_indices: Memo<Arc<[u32]>>,
) -> Element {
    #[cfg(target_arch = "wasm32")]
    let mut first_visible_row = use_signal(|| 0usize);
    #[cfg(not(target_arch = "wasm32"))]
    let first_visible_row = use_signal(|| 0usize);

    #[cfg(target_arch = "wasm32")]
    let mut viewport_height_px = use_signal(|| TABLE_VIEWPORT_FALLBACK_PX);
    #[cfg(not(target_arch = "wasm32"))]
    let viewport_height_px = use_signal(|| TABLE_VIEWPORT_FALLBACK_PX);

    let total = entries.read().len();
    let row_height_px = ROW_HEIGHT_PX_COMFORTABLE;
    let current_sort = *sort.read();
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

    let toggle_sort = move |col: SortColumn| {
        move |_: Event<MouseData>| {
            let mut s = sort.write();
            if s.col == col {
                s.dir = if s.dir == SortDir::Asc {
                    SortDir::Desc
                } else {
                    SortDir::Asc
                };
            } else {
                s.col = col;
                s.dir = SortDir::Asc;
            }
            *page.write() = 0;
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
                    if let Some(win) = web_sys::window() {
                        if let Some(document) = win.document() {
                            if let Some(node) = document.get_element_by_id(TABLE_SCROLL_ID) {
                                if let Ok(div) = node.dyn_into::<web_sys::HtmlElement>() {
                                    let top = div.scroll_top().max(0) as usize;
                                    let height = div.client_height().max(0) as usize;
                                    let next_first = (top / row_height_px).min(total);
                                    if next_first != *first_visible_row.read() {
                                        *first_visible_row.write() = next_first;
                                    }
                                    if height > 0 && height != *viewport_height_px.read() {
                                        *viewport_height_px.write() = height;
                                    }
                                }
                            }
                        }
                    }
                }
            },
            table {
                class: "results-table",
                aria_label: "{t(locale, TextKey::TableTriplesAria)}",
                thead {
                    tr {
                        th { class: "th-static", scope: "col",
                            "{t(locale, TextKey::Structure)}"
                        }
                        th {
                            class: "sort-th",
                            scope: "col",
                            aria_sort: "{aria_sort_for(&current_sort, SortColumn::Name)}",
                            onclick: toggle_sort(SortColumn::Name),
                            "{t(locale, TextKey::Compound)} "
                            span {
                                class: "sort-icon",
                                "aria-hidden": "true",
                                {sort_icon_for(&current_sort, SortColumn::Name)}
                            }
                        }
                        th {
                            class: "sort-th",
                            scope: "col",
                            aria_sort: "{aria_sort_for(&current_sort, SortColumn::Mass)}",
                            onclick: toggle_sort(SortColumn::Mass),
                            "{t(locale, TextKey::Mass)} "
                            span {
                                class: "sort-icon",
                                "aria-hidden": "true",
                                {sort_icon_for(&current_sort, SortColumn::Mass)}
                            }
                        }
                        th {
                            class: "sort-th",
                            scope: "col",
                            aria_sort: "{aria_sort_for(&current_sort, SortColumn::Formula)}",
                            onclick: toggle_sort(SortColumn::Formula),
                            "{t(locale, TextKey::Formula)} "
                            span {
                                class: "sort-icon",
                                "aria-hidden": "true",
                                {sort_icon_for(&current_sort, SortColumn::Formula)}
                            }
                        }
                        th {
                            class: "sort-th",
                            scope: "col",
                            aria_sort: "{aria_sort_for(&current_sort, SortColumn::TaxonName)}",
                            onclick: toggle_sort(SortColumn::TaxonName),
                            "{t(locale, TextKey::TaxonCol)} "
                            span {
                                class: "sort-icon",
                                "aria-hidden": "true",
                                {sort_icon_for(&current_sort, SortColumn::TaxonName)}
                            }
                        }
                        th {
                            class: "sort-th",
                            scope: "col",
                            aria_sort: "{aria_sort_for(&current_sort, SortColumn::RefTitle)}",
                            onclick: toggle_sort(SortColumn::RefTitle),
                            "{t(locale, TextKey::Reference)} "
                            span {
                                class: "sort-icon",
                                "aria-hidden": "true",
                                {sort_icon_for(&current_sort, SortColumn::RefTitle)}
                            }
                        }
                        th {
                            class: "sort-th",
                            scope: "col",
                            aria_sort: "{aria_sort_for(&current_sort, SortColumn::PubYear)}",
                            onclick: toggle_sort(SortColumn::PubYear),
                            "{t(locale, TextKey::Year)} "
                            span {
                                class: "sort-icon",
                                "aria-hidden": "true",
                                {sort_icon_for(&current_sort, SortColumn::PubYear)}
                            }
                        }
                    }
                }
                tbody {
                    if top_spacer_px > 0 {
                        tr {
                            class: "virtual-spacer-row",
                            aria_hidden: "true",
                            td {
                                class: "virtual-spacer-cell",
                                colspan: "7",
                                style: "height: {top_spacer_px}px;",
                            }
                        }
                    }
                    {
                        // Keep a single read for each reactive source per window render.
                        let rows = entries.read();
                        let order = sorted_indices.read();
                        {visible_rows_view(locale, &rows, order.as_ref(), start_row, visible_count)}
                    }
                    if bottom_spacer_px > 0 {
                        tr {
                            class: "virtual-spacer-row",
                            aria_hidden: "true",
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

fn row_view(locale: Locale, entry: &CompoundEntry, row_key: u32) -> Element {
    // URLs can be interpolated inline in RSX — no need for intermediate
    // `String` allocations. Only DOI / depict / statement / inchikey search
    // require conditional work, so those are the only helpers we still call.
    let compound_qid = entry.compound_qid.as_str();
    let taxon_qid = entry.taxon_qid.as_str();
    let reference_qid = entry.reference_qid.as_str();
    let doi_url = entry.doi_url();
    let depict_url = entry.depict_url();
    let statement_id = entry.statement_id();
    let truncated_ref_title = entry
        .ref_title
        .as_deref()
        .map(|title| truncate_title(title, 60));
    let name: &str = if entry.name.trim().is_empty() {
        entry.compound_qid.as_str()
    } else {
        &entry.name
    };
    let inchikey_search = entry
        .inchikey
        .as_deref()
        .map(|ik| format!("https://www.wikidata.org/wiki/Special:Search?search={ik}"));

    rsx! {
        tr { key: "{row_key}", class: "data-row",
            {structure_cell(locale, depict_url.as_deref(), name)}
            {compound_cell(locale, entry, name, compound_qid, inchikey_search.as_deref())}
            {mass_cell(entry.mass)}
            {formula_cell(entry.formula.as_deref())}
            {taxon_cell(locale, entry, taxon_qid)}
            {reference_cell(
                locale,
                entry,
                reference_qid,
                doi_url.as_deref(),
                statement_id.as_deref(),
                truncated_ref_title.as_deref(),
            )}
            {year_cell(entry.pub_year)}
        }
    }
}

fn visible_rows_view(
    locale: Locale,
    rows: &[CompoundEntry],
    order: &[u32],
    start_row: usize,
    visible_count: usize,
) -> Element {
    rsx! {
        for i in order.iter().skip(start_row).take(visible_count).copied() {
            {row_view(locale, &rows[i as usize], i)}
        }
    }
}

fn structure_cell(locale: Locale, depict_url: Option<&str>, name: &str) -> Element {
    rsx! {
        td { class: "td-depict",
            if let Some(url) = depict_url {
                a {
                    href: "{url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{t(locale, TextKey::OpenFullSizeDepiction)}",
                    img {
                        class: "depict-img",
                        src: "{url}",
                        alt: "{aria_chemical_structure(locale, name)}",
                        loading: "lazy",
                        width: "120",
                        height: "72",
                    }
                }
            } else {
                span { class: "na", "—" }
            }
        }
    }
}

fn compound_cell(
    locale: Locale,
    entry: &CompoundEntry,
    name: &str,
    compound_qid: &str,
    inchikey_search: Option<&str>,
) -> Element {
    rsx! {
        td { class: "td-compound",
            div { class: "cell-primary",
                a {
                    href: "https://www.wikidata.org/entity/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "primary-link",
                    "{name}"
                }
            }
            div { class: "badge-row",
                a {
                    href: "https://www.wikidata.org/entity/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "id-badge wd",
                    title: "{t(locale, TextKey::OpenInWikidata)}",
                    aria_label: "{aria_wikidata_entity(locale, compound_qid)}",
                    "{compound_qid}"
                }
                a {
                    href: "https://scholia.toolforge.org/chemical/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "id-badge sc",
                    title: "{t(locale, TextKey::OpenInScholia)}",
                    "Scholia"
                }
                if let (Some(ik), Some(search_url)) = (entry.inchikey.as_deref(), inchikey_search) {
                    a {
                        href: "{search_url}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "id-badge mono inchikey",
                        title: "{ik}",
                        aria_label: "{aria_search_inchikey(locale, ik)}",
                        "{short_inchikey(ik)}"
                    }
                }
            }
        }
    }
}

fn mass_cell(mass: Option<f64>) -> Element {
    rsx! {
        td { class: "td-num",
            if let Some(m) = mass {
                span { "{m:.4}" }
            } else {
                span { class: "na", "—" }
            }
        }
    }
}

fn formula_cell(formula: Option<&str>) -> Element {
    rsx! {
        td { class: "td-formula",
            if let Some(f) = formula {
                span { class: "formula", "{f}" }
            } else {
                span { class: "na", "—" }
            }
        }
    }
}

fn taxon_cell(locale: Locale, entry: &CompoundEntry, taxon_qid: &str) -> Element {
    rsx! {
        td { class: "td-taxon",
            div { class: "cell-primary",
                a {
                    href: "https://www.wikidata.org/entity/{taxon_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "primary-link taxon",
                    "{entry.taxon_name}"
                }
            }
            div { class: "badge-row",
                a {
                    href: "https://www.wikidata.org/entity/{taxon_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "id-badge wd",
                    title: "{t(locale, TextKey::OpenInWikidata)}",
                    aria_label: "{aria_wikidata_entity(locale, taxon_qid)}",
                    "{taxon_qid}"
                }
            }
        }
    }
}

fn reference_cell(
    locale: Locale,
    entry: &CompoundEntry,
    reference_qid: &str,
    doi_url: Option<&str>,
    statement_id: Option<&str>,
    truncated_ref_title: Option<&str>,
) -> Element {
    rsx! {
        td { class: "td-ref",
            div { class: "cell-primary",
                if let (Some(full_title), Some(display_title)) =
                    (entry.ref_title.as_deref(), truncated_ref_title)
                {
                    a {
                        href: "https://www.wikidata.org/entity/{reference_qid}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "primary-link",
                        title: "{full_title}",
                        "{display_title}"
                    }
                } else {
                    a {
                        href: "https://www.wikidata.org/entity/{reference_qid}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "primary-link",
                        "{reference_qid}"
                    }
                }
            }
            div { class: "badge-row",
                a {
                    href: "https://www.wikidata.org/entity/{reference_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "id-badge wd",
                    title: "{t(locale, TextKey::OpenInWikidata)}",
                    aria_label: "{aria_wikidata_entity(locale, reference_qid)}",
                    "{reference_qid}"
                }
                if let Some(url) = doi_url {
                    a {
                        href: "{url}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "id-badge doi",
                        title: "{t(locale, TextKey::OpenDoi)}",
                        "DOI"
                    }
                }
                if let Some(stmt) = statement_id {
                    a {
                        href: "https://www.wikidata.org/entity/statement/{stmt}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "id-badge stmt mono",
                        title: "{stmt}",
                        aria_label: "{aria_wikidata_statement(locale, stmt)}",
                        "{t(locale, TextKey::Statement)}"
                    }
                }
            }
        }
    }
}

fn year_cell(pub_year: Option<i16>) -> Element {
    rsx! {
        td { class: "td-year",
            if let Some(y) = pub_year {
                span { "{y}" }
            } else {
                span { class: "na", "—" }
            }
        }
    }
}

fn short_inchikey(ik: &str) -> &str {
    ik.split('-').next().unwrap_or(ik)
}

/// ARIA `aria-sort` value for a column header.
fn aria_sort_for(state: &SortState, col: SortColumn) -> &'static str {
    if state.col != col {
        "none"
    } else if state.dir == SortDir::Asc {
        "ascending"
    } else {
        "descending"
    }
}

fn sort_icon_for(state: &SortState, col: SortColumn) -> &'static str {
    if state.col == col {
        if state.dir == SortDir::Asc {
            "↑"
        } else {
            "↓"
        }
    } else {
        ""
    }
}

fn truncate_title(title: &str, max_chars: usize) -> String {
    let trimmed = title.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let mut out: String = trimmed.chars().take(max_chars).collect();
    out.push('…');
    out
}
