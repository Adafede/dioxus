use crate::export;
use crate::i18n::{CountNoun, Locale, count_label, showing_rows_text};
use crate::models::*;
use crate::sparql;
use dioxus::prelude::*;
use std::sync::Arc;

const VIRTUAL_INITIAL_ROWS: usize = 120;
const VIRTUAL_STEP_ROWS: usize = 200;

/// Human-facing QLever UI endpoint (for the "Open in QLever" deep-link).
const QLEVER_UI: &str = "https://qlever.dev/wikidata";

#[component]
pub fn ResultsTable(
    /// Display rows (capped to `TABLE_ROW_LIMIT`). Passed as a `Signal` so the
    /// prop diff is identity-based (pointer compare on the generational-box id)
    /// rather than content-based — we never scan the whole Vec just to decide
    /// whether the table needs to re-render.
    entries: ReadSignal<Rows>,
    locale: Locale,
    stats: DatasetStats,
    total_stats: Option<DatasetStats>,
    total_matches: Option<usize>,
    sort: Signal<SortState>,
    page: Signal<usize>,
    sparql_query: Option<String>,
    metadata_json: Option<String>,
    query_hash: Option<String>,
    result_hash: Option<String>,
    /// Active search criteria — used to build Python-compatible download
    /// filenames (taxon slug + optional search-type suffix).
    criteria: ReadSignal<SearchCriteria>,
) -> Element {
    // JSON/TTL exports re-fetch via `execute_sparql_cached`, which returns
    // the last CSV body instantly — no need to keep a full in-memory row set.
    let display_stats = total_stats
        .as_ref()
        .cloned()
        .unwrap_or_else(|| stats.clone());
    let total = entries.read().len();
    let stats_partial = false;
    let entries_value = total_matches.unwrap_or(display_stats.n_entries);
    let _ = page;

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

    let mut visible_rows_limit = use_signal(|| VIRTUAL_INITIAL_ROWS);
    let visible_count = (*visible_rows_limit.read()).min(total);

    let sort_icon = move |col: SortColumn| -> &'static str {
        let s = *sort.read();
        if s.col == col {
            if s.dir == SortDir::Asc { "↑" } else { "↓" }
        } else {
            ""
        }
    };

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

    // ── Export filenames & URLs (memoised; only rebuild when inputs change) ──
    let export_available = sparql_query.is_some() || metadata_json.is_some();

    let csv_filename = use_memo(move || {
        let c = criteria.read();
        export::generate_filename(&c.taxon, "csv", search_type_suffix(&c))
    });
    let json_filename = use_memo(move || {
        let c = criteria.read();
        export::generate_filename(&c.taxon, "ndjson", search_type_suffix(&c))
    });
    let ttl_filename = use_memo(move || {
        let c = criteria.read();
        export::generate_filename(&c.taxon, "ttl", search_type_suffix(&c))
    });

    // Metadata filename mirrors Python: `{query_hash}_{result_hash}_metadata.json`.
    let metadata_filename = match (query_hash.as_deref(), result_hash.as_deref()) {
        (Some(q), Some(r)) => format!("{q}_{r}_metadata.json"),
        _ => {
            let c = criteria.read();
            export::generate_filename(&c.taxon, "metadata.json", search_type_suffix(&c))
        }
    };

    let qlever_ui_url = sparql_query
        .as_deref()
        .map(|q| format!("{QLEVER_UI}?query={}", urlencoding::encode(q)));
    let mut download_busy = use_signal(|| false);
    let mut download_status: Signal<Option<String>> = use_signal(|| None);
    let download_status_text = download_status
        .read()
        .clone()
        .unwrap_or_else(|| "Preparing download...".to_string());

    rsx! {
        div { class: "results-wrap",
            // ── Stats + toolbar ───────────────────────────────────────────
            div { class: "results-toolbar",
                div {
                    class: "stat-bar",
                    role: "group",
                    aria_label: "Dataset statistics",
                    StatBadge {
                        locale,
                        value: display_stats.n_compounds,
                        noun: CountNoun::Compound,
                        plus: stats_partial,
                    }
                    StatBadge {
                        locale,
                        value: display_stats.n_taxa,
                        noun: CountNoun::Taxon,
                        plus: stats_partial,
                    }
                    StatBadge {
                        locale,
                        value: display_stats.n_references,
                        noun: CountNoun::Reference,
                        plus: stats_partial,
                    }
                    StatBadge {
                        locale,
                        value: entries_value,
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
                            aria_label: "Download results",
                            if let Some(query) = sparql_query.as_deref() {
                                button {
                                    class: "btn btn-sm",
                                    r#type: "button",
                                    disabled: *download_busy.read(),
                                    onclick: {
                                        let q = query.to_string();
                                        move |_| {
                                            let filename = csv_filename.read().clone();
                                            *download_busy.write() = true;
                                            *download_status.write() = Some("Starting CSV download...".to_string());
                                            trigger_query_csv_download(&q, &filename);
                                            *download_busy.write() = false;
                                            *download_status.write() = None;
                                        }
                                    },
                                    title: "Download all rows as CSV",
                                    "CSV"
                                }
                                button {
                                    class: "btn btn-sm",
                                    r#type: "button",
                                    disabled: *download_busy.read(),
                                    onclick: {
                                        let q = query.to_string();
                                        move |_| {
                                            let q = q.clone();
                                            let filename = json_filename.read().clone();
                                            *download_busy.write() = true;
                                            *download_status.write() = Some("Preparing JSON download...".to_string());
                                            spawn(async move {
                                                if let Ok(csv) = sparql::execute_sparql_cached(&q).await {
                                                    if let Ok(rows) = sparql::parse_compounds_csv(csv.as_str()) {
                                                        let body = export::build_ndjson(&rows);
                                                        trigger_download(&filename, "application/x-ndjson", &body);
                                                    }
                                                }
                                                *download_busy.write() = false;
                                                *download_status.write() = None;
                                            });
                                        }
                                    },
                                    title: "Download all rows as newline-delimited JSON (can take time)",
                                    "JSON"
                                }
                                button {
                                    class: "btn btn-sm",
                                    r#type: "button",
                                    disabled: *download_busy.read(),
                                    onclick: {
                                        let q = query.to_string();
                                        let query_hash_value = query_hash.clone();
                                        let result_hash_value = result_hash.clone();
                                        move |_| {
                                            let q = q.clone();
                                            let filename = ttl_filename.read().clone();
                                            let qh = query_hash_value.clone();
                                            let rh = result_hash_value.clone();
                                            let crit = criteria.peek().clone();
                                            *download_busy.write() = true;
                                            *download_status.write() = Some("Preparing TTL download...".to_string());
                                            spawn(async move {
                                                if let Ok(csv) = sparql::execute_sparql_cached(&q).await {
                                                    if let Ok(rows) = sparql::parse_compounds_csv(csv.as_str()) {
                                                        let full_stats = DatasetStats::from_entries(&rows);
                                                        let ttl = export::build_ttl(
                                                            &rows,
                                                            export::MetadataInputs {
                                                                criteria: &crit,
                                                                qid: None,
                                                                stats: &full_stats,
                                                                number_of_records_override: Some(full_stats.n_entries),
                                                                query_hash: qh.as_deref().unwrap_or(""),
                                                                result_hash: rh.as_deref().unwrap_or(""),
                                                            },
                                                        );
                                                        trigger_download(&filename, "text/turtle", &ttl);
                                                    }
                                                }
                                                *download_busy.write() = false;
                                                *download_status.write() = None;
                                            });
                                        }
                                    },
                                    title: "Download all rows as RDF Turtle (can take time)",
                                    "TTL"
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
                                        move |_| trigger_download(&filename, "application/ld+json", &body)
                                    },
                                    title: "Download Schema.org metadata (JSON-LD)",
                                    "Metadata"
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
                            title: "Open this query in the QLever web interface",
                            "Open in QLever"
                        }
                    }
                }
            }

            if let Some(q) = sparql_query.as_deref() {
                details { class: "query-panel",
                    summary { "SPARQL query" }
                    div { class: "query-panel-actions",
                        crate::components::copy_button::CopyButton { text: q.to_string(), title: "Copy SPARQL query" }
                    }
                    pre { class: "query-text", "{q}" }
                }
            }

            if total == 0 {
                div { class: "empty-state",
                    p { "No results. Try broadening your search." }
                }
            } else {
                div { class: "pagination-bar",
                    span { class: "page-info", "{showing_rows_text(locale, visible_count, total)}" }
                    if visible_count < total {
                        button {
                            class: "btn btn-sm",
                            r#type: "button",
                            onclick: move |_| {
                                let next = (*visible_rows_limit.peek()).saturating_add(VIRTUAL_STEP_ROWS);
                                *visible_rows_limit.write() = next;
                            },
                            "Load {VIRTUAL_STEP_ROWS} more"
                        }
                    }
                }

                div { class: "table-scroll",
                    table {
                        class: "results-table",
                        aria_label: "Compound–taxon–reference triples",
                        thead {
                            tr {
                                th { class: "th-static", scope: "col", "Structure" }
                                th {
                                    class: "sort-th",
                                    scope: "col",
                                    aria_sort: "{aria_sort_for(&sort.read(), SortColumn::Name)}",
                                    onclick: toggle_sort(SortColumn::Name),
                                    "Compound "
                                    span {
                                        class: "sort-icon",
                                        "aria-hidden": "true",
                                        {sort_icon(SortColumn::Name)}
                                    }
                                }
                                th {
                                    class: "sort-th",
                                    scope: "col",
                                    aria_sort: "{aria_sort_for(&sort.read(), SortColumn::Mass)}",
                                    onclick: toggle_sort(SortColumn::Mass),
                                    "Mass "
                                    span {
                                        class: "sort-icon",
                                        "aria-hidden": "true",
                                        {sort_icon(SortColumn::Mass)}
                                    }
                                }
                                th {
                                    class: "sort-th",
                                    scope: "col",
                                    aria_sort: "{aria_sort_for(&sort.read(), SortColumn::Formula)}",
                                    onclick: toggle_sort(SortColumn::Formula),
                                    "Formula "
                                    span {
                                        class: "sort-icon",
                                        "aria-hidden": "true",
                                        {sort_icon(SortColumn::Formula)}
                                    }
                                }
                                th {
                                    class: "sort-th",
                                    scope: "col",
                                    aria_sort: "{aria_sort_for(&sort.read(), SortColumn::TaxonName)}",
                                    onclick: toggle_sort(SortColumn::TaxonName),
                                    "Taxon "
                                    span {
                                        class: "sort-icon",
                                        "aria-hidden": "true",
                                        {sort_icon(SortColumn::TaxonName)}
                                    }
                                }
                                th {
                                    class: "sort-th",
                                    scope: "col",
                                    aria_sort: "{aria_sort_for(&sort.read(), SortColumn::RefTitle)}",
                                    onclick: toggle_sort(SortColumn::RefTitle),
                                    "Reference "
                                    span {
                                        class: "sort-icon",
                                        "aria-hidden": "true",
                                        {sort_icon(SortColumn::RefTitle)}
                                    }
                                }
                                th {
                                    class: "sort-th",
                                    scope: "col",
                                    aria_sort: "{aria_sort_for(&sort.read(), SortColumn::PubYear)}",
                                    onclick: toggle_sort(SortColumn::PubYear),
                                    "Year "
                                    span {
                                        class: "sort-icon",
                                        "aria-hidden": "true",
                                        {sort_icon(SortColumn::PubYear)}
                                    }
                                }
                            }
                        }
                        tbody {
                            {
                                let rows = entries.read();
                                let order = sorted_indices.read();
                                rsx! {
                                    for i in order.iter().take(visible_count).copied() {
                                        Row { key: "{i}", entry: rows[i as usize].clone() }
                                    }
                                }
                            }
                        }
                    }
                }

                if visible_count < total {
                    div { class: "pagination-bar",
                        button {
                            class: "btn btn-sm",
                            r#type: "button",
                            onclick: move |_| {
                                let next = (*visible_rows_limit.peek()).saturating_add(VIRTUAL_STEP_ROWS);
                                *visible_rows_limit.write() = next;
                            },
                            "Load more"
                        }
                    }
                }
            }
        }
    }
}

fn search_type_suffix(criteria: &SearchCriteria) -> Option<&'static str> {
    if criteria.smiles.trim().is_empty() {
        None
    } else {
        Some(match criteria.smiles_search_type {
            SmilesSearchType::Substructure => "substructure",
            SmilesSearchType::Similarity => "similarity",
        })
    }
}

// ── Sub-components ────────────────────────────────────────────────────────────

#[component]
fn StatBadge(locale: Locale, value: usize, noun: CountNoun, plus: bool) -> Element {
    let display_value = if plus {
        format!("{value}+")
    } else {
        value.to_string()
    };
    let label = count_label(locale, noun, value);
    rsx! {
        div { class: "stat-badge",
            span { class: "stat-value", "{display_value}" }
            " "
            span { class: "stat-label", "{label}" }
        }
    }
}

#[component]
fn Row(entry: CompoundEntry) -> Element {
    // URLs can be interpolated inline in RSX — no need for intermediate
    // `String` allocations. Only DOI / depict / statement / inchikey search
    // require conditional work, so those are the only helpers we still call.
    let compound_qid = entry.compound_qid.clone();
    let taxon_qid = entry.taxon_qid.clone();
    let reference_qid = entry.reference_qid.clone();
    let doi_url = entry.doi_url();
    let depict_url = entry.depict_url();
    let statement_id = entry.statement_id();
    let name: &str = if entry.name.trim().is_empty() {
        entry.compound_qid.as_str()
    } else {
        entry.name.as_str()
    };
    let inchikey_search = entry
        .inchikey
        .as_deref()
        .map(|ik| format!("https://www.wikidata.org/wiki/Special:Search?search={ik}"));

    rsx! {
        tr { class: "data-row",
            // ── Structure depiction ─────────────────────────────────────────
            td { class: "td-depict",
                if let Some(url) = depict_url {
                    a {
                        href: "{url}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        title: "Open full-size depiction",
                        img {
                            class: "depict-img",
                            src: "{url}",
                            alt: "Chemical structure of {name}",
                            loading: "lazy",
                            width: "120",
                            height: "72",
                        }
                    }
                } else {
                    span { class: "na", "—" }
                }
            }

            // ── Compound: bold name link + QID / Scholia / InChIKey badges ──
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
                        title: "Open in Wikidata",
                        aria_label: "Wikidata {compound_qid}",
                        "{compound_qid}"
                    }
                    a {
                        href: "https://scholia.toolforge.org/chemical/{compound_qid}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "id-badge sc",
                        title: "Open in Scholia",
                        "Scholia"
                    }
                    if let (Some(ik), Some(search_url)) = (
                        entry.inchikey.as_deref(),
                        inchikey_search.as_deref(),
                    )
                    {
                        a {
                            href: "{search_url}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "id-badge mono inchikey",
                            title: "{ik}",
                            aria_label: "Search Wikidata for InChIKey {ik}",
                            "{short_inchikey(ik)}"
                        }
                    }
                }
            }

            // ── Mass ────────────────────────────────────────────────────────
            td { class: "td-num",
                if let Some(m) = entry.mass {
                    span { "{m:.4}" }
                } else {
                    span { class: "na", "—" }
                }
            }
            // ── Formula ─────────────────────────────────────────────────────
            td { class: "td-formula",
                if let Some(f) = entry.formula.as_deref() {
                    span { class: "formula", "{f}" }
                } else {
                    span { class: "na", "—" }
                }
            }

            // ── Taxon: italic name link + QID badge ────────────────────────
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
                        title: "Open in Wikidata",
                        aria_label: "Wikidata {taxon_qid}",
                        "{taxon_qid}"
                    }
                }
            }

            // ── Reference: title link + QID / DOI / statement badges ────────
            td { class: "td-ref",
                div { class: "cell-primary",
                    if let Some(t) = entry.ref_title.as_deref() {
                        a {
                            href: "https://www.wikidata.org/entity/{reference_qid}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "primary-link",
                            title: "{t}",
                            "{truncate_title(t, 60)}"
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
                        title: "Open in Wikidata",
                        aria_label: "Wikidata {reference_qid}",
                        "{reference_qid}"
                    }
                    if let Some(url) = doi_url {
                        a {
                            href: "{url}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "id-badge doi",
                            title: "Open DOI",
                            "DOI"
                        }
                    }
                    if let Some(stmt) = statement_id.as_deref() {
                        a {
                            href: "https://www.wikidata.org/entity/statement/{stmt}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "id-badge stmt mono",
                            title: "{stmt}",
                            aria_label: "Wikidata statement {stmt}",
                            "statement"
                        }
                    }
                }
            }

            // ── Year ────────────────────────────────────────────────────────
            td { class: "td-year",
                if let Some(y) = entry.pub_year {
                    span { "{y}" }
                } else {
                    span { class: "na", "—" }
                }
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

fn truncate_title(title: &str, max_chars: usize) -> String {
    let trimmed = title.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let mut out: String = trimmed.chars().take(max_chars).collect();
    out.push('…');
    out
}

// ── Download helpers ──────────────────────────────────────────────────────────

fn trigger_download(filename: &str, mime: &str, body: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let filename_json =
            serde_json::to_string(filename).unwrap_or_else(|_| "\"download.txt\"".to_string());
        let mime_json = serde_json::to_string(mime)
            .unwrap_or_else(|_| "\"application/octet-stream\"".to_string());
        let body_json = serde_json::to_string(body).unwrap_or_else(|_| "\"\"".to_string());
        let script = format!(
            r#"(() => {{
  const filename = {filename_json};
  const mime = {mime_json};
  const content = {body_json};
  const blob = new Blob([content], {{ type: mime }});
  const url = URL.createObjectURL(blob);
  const nav = window.navigator;
  const ua = nav.userAgent || "";
  const isIOS = /iPad|iPhone|iPod/.test(ua) || (nav.platform === "MacIntel" && nav.maxTouchPoints > 1);
  if (isIOS) {{
    window.open(url, "_blank", "noopener,noreferrer");
    setTimeout(() => URL.revokeObjectURL(url), 60_000);
    return;
  }}
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.rel = "noopener noreferrer";
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 0);
}})();"#
        );
        let _ = js_sys::eval(&script);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (filename, mime, body);
    }
}

fn trigger_query_csv_download(sparql_query: &str, filename: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let endpoint_json = serde_json::to_string("https://qlever.dev/api/wikidata")
            .unwrap_or_else(|_| "\"https://qlever.dev/api/wikidata\"".to_string());
        let query_json = serde_json::to_string(sparql_query).unwrap_or_else(|_| "\"\"".to_string());
        let filename_json =
            serde_json::to_string(filename).unwrap_or_else(|_| "\"download.csv\"".to_string());
        let script = format!(
            r#"(() => {{
  const endpoint = {endpoint_json};
  const query = {query_json};
  const filename = {filename_json};
  const body = new URLSearchParams();
  body.set("query", query);
  body.set("action", "csv_export");

  fetch(endpoint, {{
    method: "POST",
    headers: {{ "Accept": "text/csv" }},
    body,
  }})
    .then((r) => {{ if (!r.ok) throw new Error(`HTTP ${{r.status}}`); return r.blob(); }})
    .then((blob) => {{
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = filename;
      a.rel = "noopener noreferrer";
      document.body.appendChild(a);
      a.click();
      a.remove();
      setTimeout(() => URL.revokeObjectURL(url), 0);
    }})
    .catch(() => {{
      const ui = "https://qlever.dev/wikidata?query=" + encodeURIComponent(query);
      window.open(ui, "_blank", "noopener,noreferrer");
    }});
}})();"#
        );
        let _ = js_sys::eval(&script);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (sparql_query, filename);
    }
}
