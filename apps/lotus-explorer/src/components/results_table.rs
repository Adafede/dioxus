use crate::export;
use crate::models::*;
use crate::sparql;
use dioxus::prelude::*;

const VIRTUAL_INITIAL_ROWS: usize = 120;
const VIRTUAL_STEP_ROWS: usize = 200;

/// Human-facing QLever UI endpoint (for the "Open in QLever" deep-link).
/// The JSON API lives at `https://qlever.dev/api/wikidata`; the interactive
/// UI is one path up at `https://qlever.dev/wikidata`.
const QLEVER_UI: &str = "https://qlever.dev/wikidata";

#[component]
pub fn ResultsTable(
    entries: Vec<CompoundEntry>,
    /// Full filtered dataset — exports run over this so the files contain
    /// every row even when the on-screen table is display-capped.
    export_rows: Vec<CompoundEntry>,
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
    criteria: SearchCriteria,
) -> Element {
    let display_stats = total_stats.unwrap_or_else(|| stats.clone());
    let stats_partial = total_matches
        .map(|n| n > entries.len())
        .unwrap_or(entries.len() >= TABLE_ROW_LIMIT);
    let entries_value = total_matches.unwrap_or(display_stats.n_entries);
    let total = entries.len();
    let _ = page;

    // Sort
    let mut sorted = entries.clone();
    {
        let s = sort.read();
        sorted.sort_by(|a, b| {
            let cmp = match s.col {
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Mass => a
                    .mass
                    .partial_cmp(&b.mass)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Formula => a.formula.cmp(&b.formula),
                SortColumn::TaxonName => a.taxon_name.cmp(&b.taxon_name),
                SortColumn::PubYear => a.pub_year.cmp(&b.pub_year),
                SortColumn::RefTitle => a.ref_title.cmp(&b.ref_title),
            };
            if s.dir == SortDir::Desc {
                cmp.reverse()
            } else {
                cmp
            }
        });
    }

    let mut visible_rows_limit = use_signal(|| VIRTUAL_INITIAL_ROWS);
    let visible_count = (*visible_rows_limit.read()).min(sorted.len());
    let visible_rows: Vec<CompoundEntry> = sorted.iter().take(visible_count).cloned().collect();

    let sort_icon = |col: SortColumn| -> &'static str {
        let s = sort.read();
        if s.col == col {
            if s.dir == SortDir::Asc { "↑" } else { "↓" }
        } else {
            ""
        }
    };

    let toggle_sort = |col: SortColumn| {
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

    // ── Export (runs over the *full* filtered set, not just the visible page) ──
    let export_available = sparql_query.is_some() || metadata_json.is_some();
    // Python-compatible filenames: {YYYYMMDD}_lotus_{safe_taxon}[_{search_type}].{ext}
    let search_type_suffix: Option<&str> = if criteria.smiles.trim().is_empty() {
        None
    } else {
        Some(match criteria.smiles_search_type {
            SmilesSearchType::Substructure => "substructure",
            SmilesSearchType::Similarity => "similarity",
        })
    };
    let csv_filename = export::generate_filename(&criteria.taxon, "csv", search_type_suffix);
    let json_filename = export::generate_filename(&criteria.taxon, "ndjson", search_type_suffix);
    let ttl_filename = export::generate_filename(&criteria.taxon, "ttl", search_type_suffix);
    // Metadata filename mirrors Python: `{query_hash}_{result_hash}_metadata.json`.
    let metadata_filename = match (query_hash.as_deref(), result_hash.as_deref()) {
        (Some(q), Some(r)) => format!("{q}_{r}_metadata.json"),
        _ => export::generate_filename(&criteria.taxon, "metadata.json", search_type_suffix),
    };
    let metadata_body = metadata_json.clone();
    let qlever_ui_url = sparql_query
        .as_deref()
        .map(|q| format!("{QLEVER_UI}?query={}", urlencoding::encode(q)));
    let download_busy = use_signal(|| false);
    let download_status: Signal<Option<String>> = use_signal(|| None);
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
                        value: display_stats.n_compounds,
                        label: "Compounds",
                        plus: stats_partial,
                    }
                    StatBadge {
                        value: display_stats.n_taxa,
                        label: "Taxa",
                        plus: stats_partial,
                    }
                    StatBadge {
                        value: display_stats.n_references,
                        label: "References",
                        plus: stats_partial,
                    }
                    StatBadge { value: entries_value, label: "Entries", plus: false }
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
                                        let filename = csv_filename.clone();
                                        let q = query.to_string();
                                        let mut busy = download_busy;
                                        let mut status = download_status;
                                        move |_| {
                                            *busy.write() = true;
                                            *status.write() = Some("Starting CSV download...".to_string());
                                            trigger_query_csv_download(&q, &filename);
                                            *busy.write() = false;
                                            *status.write() = None;
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
                                        let filename = json_filename.clone();
                                        let mut busy = download_busy;
                                        let mut status = download_status;
                                        move |_| {
                                            *busy.write() = true;
                                            *status.write() = Some("Preparing JSON download...".to_string());
                                            spawn({
                                                let q = q.clone();
                                                let filename = filename.clone();
                                                let mut busy = busy;
                                                let mut status = status;
                                                async move {
                                                    if let Ok(csv) = sparql::execute_sparql(&q).await {
                                                        if let Ok(rows) = sparql::parse_compounds_csv(&csv) {
                                                            let body = export::build_ndjson(&rows);
                                                            trigger_download(&filename, "application/x-ndjson", &body);
                                                        }
                                                    }
                                                    *busy.write() = false;
                                                    *status.write() = None;
                                                }
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
                                        let filename = ttl_filename.clone();
                                        let query_hash_value = query_hash.clone();
                                        let result_hash_value = result_hash.clone();
                                        let criteria_value = criteria.clone();
                                        let mut busy = download_busy;
                                        let mut status = download_status;
                                        move |_| {
                                            *busy.write() = true;
                                            *status.write() = Some("Preparing TTL download...".to_string());
                                            spawn({
                                                let q = q.clone();
                                                let filename = filename.clone();
                                                let query_hash_value = query_hash_value.clone();
                                                let result_hash_value = result_hash_value.clone();
                                                let criteria_value = criteria_value.clone();
                                                let mut busy = busy;
                                                let mut status = status;
                                                async move {
                                                    if let Ok(csv) = sparql::execute_sparql(&q).await {
                                                        if let Ok(rows) = sparql::parse_compounds_csv(&csv) {
                                                            let full_stats = DatasetStats::from_entries(&rows);
                                                            let ttl = export::build_ttl(
                                                                &rows,
                                                                export::MetadataInputs {
                                                                    criteria: &criteria_value,
                                                                    qid: None,
                                                                    stats: &full_stats,
                                                                    number_of_records_override:
                                                                        Some(full_stats.n_entries),
                                                                    query_hash: query_hash_value
                                                                        .as_deref()
                                                                        .unwrap_or(""),
                                                                    result_hash: result_hash_value
                                                                        .as_deref()
                                                                        .unwrap_or(""),
                                                                },
                                                            );
                                                            trigger_download(&filename, "text/turtle", &ttl);
                                                        }
                                                    }
                                                    *busy.write() = false;
                                                    *status.write() = None;
                                                }
                                            });
                                        }
                                    },
                                    title: "Download all rows as RDF Turtle (can take time)",
                                    "TTL"
                                }
                            }
                            if let Some(body) = metadata_body.as_ref() {
                                button {
                                    class: "btn btn-sm",
                                    r#type: "button",
                                    disabled: *download_busy.read(),
                                    onclick: {
                                        let filename = metadata_filename.clone();
                                        let body = body.clone();
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
                    pre { class: "query-text", "{q}" }
                }
            }

            if total == 0 {
                div { class: "empty-state",
                    p { "No results. Try broadening your search." }
                }
            } else {
                div { class: "pagination-bar",
                    span { class: "page-info", "Showing {visible_count} of {total} rows" }
                    if visible_count < total {
                        button {
                            class: "btn btn-sm",
                            r#type: "button",
                            onclick: move |_| {
                                let next = (*visible_rows_limit.read()).saturating_add(VIRTUAL_STEP_ROWS);
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
                            for entry in visible_rows.iter() {
                                Row {
                                    key: "{entry.compound_qid}-{entry.taxon_qid}-{entry.reference_qid}",
                                    entry: entry.clone(),
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
                                let next = (*visible_rows_limit.read()).saturating_add(VIRTUAL_STEP_ROWS);
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

// ── Sub-components ────────────────────────────────────────────────────────────

#[component]
fn StatBadge(value: usize, label: &'static str, plus: bool) -> Element {
    let display_value = if plus {
        format!("{value}+")
    } else {
        value.to_string()
    };
    rsx! {
        div { class: "stat-badge",
            span { class: "stat-value", "{display_value}" }
            span { class: "stat-label", "{label}" }
        }
    }
}

// Pagination is intentionally replaced by chunked rendering to keep the DOM
// small while still allowing users to progressively load large result sets.

#[component]
fn Row(entry: CompoundEntry) -> Element {
    let wu = entry.compound_url();
    let tu = entry.taxon_url();
    let ru = entry.reference_url();
    let su = entry.scholia_url();
    let doi = entry.doi_url();
    let depict = entry.depict_url();
    let statement_url = entry.statement_url();
    let statement_id = entry.statement_id();
    let name = if entry.name.trim().is_empty() {
        entry.compound_qid.clone()
    } else {
        entry.name.clone()
    };
    let inchikey_search = entry
        .inchikey
        .as_ref()
        .map(|ik| format!("https://www.wikidata.org/wiki/Special:Search?search={ik}"));

    rsx! {
        tr { class: "data-row",
            // ── Structure depiction ─────────────────────────────────────────
            td { class: "td-depict",
                if let Some(url) = depict {
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
                        href: "{wu}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "primary-link",
                        "{name}"
                    }
                }
                div { class: "badge-row",
                    a {
                        href: "{wu}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "id-badge wd",
                        title: "Open in Wikidata",
                        aria_label: "Wikidata {entry.compound_qid}",
                        "{entry.compound_qid}"
                    }
                    a {
                        href: "{su}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "id-badge sc",
                        title: "Open in Scholia",
                        "Scholia"
                    }
                    if let (Some(ik), Some(search_url)) = (&entry.inchikey, inchikey_search.as_deref()) {
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
                if let Some(f) = &entry.formula {
                    span { class: "formula", "{f}" }
                } else {
                    span { class: "na", "—" }
                }
            }

            // ── Taxon: italic name link + QID badge ────────────────────────
            td { class: "td-taxon",
                div { class: "cell-primary",
                    a {
                        href: "{tu}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "primary-link taxon",
                        "{entry.taxon_name}"
                    }
                }
                div { class: "badge-row",
                    a {
                        href: "{tu}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "id-badge wd",
                        title: "Open in Wikidata",
                        aria_label: "Wikidata {entry.taxon_qid}",
                        "{entry.taxon_qid}"
                    }
                }
            }

            // ── Reference: title link + QID / DOI / statement badges ────────
            td { class: "td-ref",
                div { class: "cell-primary",
                    if let Some(t) = &entry.ref_title {
                        a {
                            href: "{ru}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "primary-link",
                            title: "{t}",
                            "{truncate_title(t, 60)}"
                        }
                    } else {
                        a {
                            href: "{ru}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "primary-link",
                            "{entry.reference_qid}"
                        }
                    }
                }
                div { class: "badge-row",
                    a {
                        href: "{ru}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "id-badge wd",
                        title: "Open in Wikidata",
                        aria_label: "Wikidata {entry.reference_qid}",
                        "{entry.reference_qid}"
                    }
                    if let Some(url) = doi {
                        a {
                            href: "{url}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "id-badge doi",
                            title: "Open DOI",
                            "DOI"
                        }
                    }
                    if let (Some(url), Some(stmt)) = (statement_url.as_deref(), statement_id.as_deref()) {
                        a {
                            href: "{url}",
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

fn short_inchikey(ik: &str) -> String {
    // Show only the 14-char connectivity hash (first block) so the badge stays compact.
    ik.split('-').next().unwrap_or(ik).to_string()
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

// ── CSV export ────────────────────────────────────────────────────────────────

/// Build a standalone CSV document for the full filtered dataset. Column
/// order mirrors the Python `build_display_dataframe` export layout.
#[allow(dead_code)]
fn build_csv(rows: &[CompoundEntry]) -> String {
    let mut out = String::with_capacity(rows.len() * 256);
    out.push_str(
        "compound_qid,compound_name,compound_inchikey,compound_smiles,compound_mass,\
         compound_formula,taxon_qid,taxon_name,reference_qid,reference_title,\
         reference_doi,reference_year,statement_id\n",
    );
    for e in rows {
        let mass_str = e.mass.map(|m| format!("{m}")).unwrap_or_default();
        let year_str = e.pub_year.map(|y| y.to_string()).unwrap_or_default();
        let stmt_str = e.statement_id().unwrap_or_default();
        let fields: [&str; 13] = [
            e.compound_qid.as_str(),
            e.name.as_str(),
            e.inchikey.as_deref().unwrap_or(""),
            e.smiles.as_deref().unwrap_or(""),
            mass_str.as_str(),
            e.formula.as_deref().unwrap_or(""),
            e.taxon_qid.as_str(),
            e.taxon_name.as_str(),
            e.reference_qid.as_str(),
            e.ref_title.as_deref().unwrap_or(""),
            e.ref_doi.as_deref().unwrap_or(""),
            year_str.as_str(),
            stmt_str.as_str(),
        ];
        for (i, f) in fields.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&csv_escape(f));
        }
        out.push('\n');
    }
    out
}

#[allow(dead_code)]
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

fn trigger_download(filename: &str, mime: &str, body: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let filename_json =
            serde_json::to_string(filename).unwrap_or_else(|_| "\"download.txt\"".to_string());
        let mime_json = serde_json::to_string(mime)
            .unwrap_or_else(|_| "\"application/octet-stream\"".to_string());
        let body_json = serde_json::to_string(body).unwrap_or_else(|_| "\"\"".to_string());
        // iOS/Orion often ignore `a[download]` for data URLs. Blob URLs with
        // an explicit iOS fallback (open in a new tab) are more reliable.
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
        // Fetch full CSV via POST and force the app's filename convention.
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
      // Fallback: at least open QLever UI with query if fetch/download fails.
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
