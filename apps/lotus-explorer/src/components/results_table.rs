use dioxus::prelude::*;
use crate::export;
use crate::models::*;

const PAGE_SIZE: usize = 10;

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
    stats:   DatasetStats,
    sort:    Signal<SortState>,
    page:    Signal<usize>,
    sparql_query: Option<String>,
    metadata_json: Option<String>,
    query_hash: Option<String>,
    result_hash: Option<String>,
    /// Active search criteria — used to build Python-compatible download
    /// filenames (taxon slug + optional search-type suffix).
    criteria: SearchCriteria,
) -> Element {
    let total = entries.len();
    let total_pages = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let cur = (*page.read()).min(total_pages - 1);

    // Sort
    let mut sorted = entries.clone();
    {
        let s = sort.read();
        sorted.sort_by(|a, b| {
            let cmp = match s.col {
                SortColumn::Name      => a.name.cmp(&b.name),
                SortColumn::Mass      => a.mass.partial_cmp(&b.mass).unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Formula   => a.formula.cmp(&b.formula),
                SortColumn::TaxonName => a.taxon_name.cmp(&b.taxon_name),
                SortColumn::PubYear   => a.pub_year.cmp(&b.pub_year),
                SortColumn::RefTitle  => a.ref_title.cmp(&b.ref_title),
            };
            if s.dir == SortDir::Desc { cmp.reverse() } else { cmp }
        });
    }

    let page_rows: Vec<CompoundEntry> = sorted
        .iter()
        .skip(cur * PAGE_SIZE)
        .take(PAGE_SIZE)
        .cloned()
        .collect();

    let sort_icon = |col: SortColumn| -> &'static str {
        let s = sort.read();
        if s.col == col { if s.dir == SortDir::Asc { "↑" } else { "↓" } } else { "" }
    };

    let toggle_sort = |col: SortColumn| {
        move |_: Event<MouseData>| {
            let mut s = sort.write();
            if s.col == col {
                s.dir = if s.dir == SortDir::Asc { SortDir::Desc } else { SortDir::Asc };
            } else {
                s.col = col;
                s.dir = SortDir::Asc;
            }
            *page.write() = 0;
        }
    };

    // ── Export (runs over the *full* filtered set, not just the visible page) ──
    let export_available = !export_rows.is_empty();
    // Python-compatible filenames: {YYYYMMDD}_lotus_{safe_taxon}[_{search_type}].{ext}
    let search_type_suffix: Option<&str> = if criteria.smiles.trim().is_empty() {
        None
    } else {
        Some(match criteria.smiles_search_type {
            SmilesSearchType::Substructure => "substructure",
            SmilesSearchType::Similarity   => "similarity",
        })
    };
    let csv_filename      = export::generate_filename(&criteria.taxon, "csv",    search_type_suffix);
    let json_filename     = export::generate_filename(&criteria.taxon, "ndjson", search_type_suffix);
    let ttl_filename      = export::generate_filename(&criteria.taxon, "ttl",    search_type_suffix);
    // Metadata filename mirrors Python: `{query_hash}_{result_hash}_metadata.json`.
    let metadata_filename = match (query_hash.as_deref(), result_hash.as_deref()) {
        (Some(q), Some(r)) => format!("{q}_{r}_metadata.json"),
        _ => export::generate_filename(&criteria.taxon, "metadata.json", search_type_suffix),
    };
    let csv_url  = export_available.then(|| export::to_data_url("text/csv", &build_csv(&export_rows)));
    let json_url = export_available.then(|| export::to_data_url("application/x-ndjson", &export::build_ndjson(&export_rows)));
    let ttl_url  = export_available
        .then(|| {
            let meta = export::MetadataInputs {
                criteria: &SearchCriteria::default(), // filled in for header-only fields
                qid: None,
                stats: &stats,
                query_hash: query_hash.as_deref().unwrap_or(""),
                result_hash: result_hash.as_deref().unwrap_or(""),
            };
            export::to_data_url("text/turtle", &export::build_ttl(&export_rows, meta))
        });
    let metadata_url = metadata_json
        .as_deref()
        .map(|m| export::to_data_url("application/ld+json", m));
    let qlever_ui_url = sparql_query
        .as_deref()
        .map(|q| format!("{QLEVER_UI}?query={}", urlencoding::encode(q)));

    rsx! {
        div { class: "results-wrap",
            // ── Stats + toolbar ───────────────────────────────────────────
            div { class: "results-toolbar",
                div { class: "stat-bar", role: "group", aria_label: "Dataset statistics",
                    StatBadge { value: stats.n_compounds,  label: "Compounds"  }
                    StatBadge { value: stats.n_taxa,       label: "Taxa"       }
                    StatBadge { value: stats.n_references, label: "References" }
                    StatBadge { value: stats.n_entries,    label: "Entries"    }
                }
                div { class: "toolbar-actions",
                    if export_available {
                        div { class: "dl-group", role: "group", aria_label: "Download results",
                            if let Some(url) = csv_url.as_deref() {
                                a { class: "btn btn-sm", href: "{url}",
                                    download: "{csv_filename}",
                                    title: "Download the full result set ({export_rows.len()} rows) as CSV",
                                    "CSV" }
                            }
                            if let Some(url) = json_url.as_deref() {
                                a { class: "btn btn-sm", href: "{url}",
                                    download: "{json_filename}",
                                    title: "Download the full result set as newline-delimited JSON",
                                    "JSON" }
                            }
                            if let Some(url) = ttl_url.as_deref() {
                                a { class: "btn btn-sm", href: "{url}",
                                    download: "{ttl_filename}",
                                    title: "Download the full result set as RDF Turtle",
                                    "TTL" }
                            }
                            if let Some(url) = metadata_url.as_deref() {
                                a { class: "btn btn-sm", href: "{url}",
                                    download: "{metadata_filename}",
                                    title: "Download Schema.org metadata (JSON-LD)",
                                    "Metadata" }
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
                Pager { cur, total_pages, total,
                    on_prev: move |_| { let mut p = page.write(); if *p > 0 { *p -= 1; } },
                    on_next: move |_| { let mut p = page.write(); if *p + 1 < total_pages { *p += 1; } } }

                div { class: "table-scroll",
                    table { class: "results-table", aria_label: "Compound–taxon–reference triples",
                        thead {
                            tr {
                                th { class: "th-static", scope: "col", "Structure" }
                                th { class: "sort-th",   scope: "col",
                                    aria_sort: "{aria_sort_for(&sort.read(), SortColumn::Name)}",
                                    onclick: toggle_sort(SortColumn::Name),
                                    "Compound " span { class: "sort-icon", "aria-hidden": "true", {sort_icon(SortColumn::Name)} }
                                }
                                th { class: "sort-th",   scope: "col",
                                    aria_sort: "{aria_sort_for(&sort.read(), SortColumn::Mass)}",
                                    onclick: toggle_sort(SortColumn::Mass),
                                    "Mass "     span { class: "sort-icon", "aria-hidden": "true", {sort_icon(SortColumn::Mass)} }
                                }
                                th { class: "sort-th",   scope: "col",
                                    aria_sort: "{aria_sort_for(&sort.read(), SortColumn::Formula)}",
                                    onclick: toggle_sort(SortColumn::Formula),
                                    "Formula "  span { class: "sort-icon", "aria-hidden": "true", {sort_icon(SortColumn::Formula)} }
                                }
                                th { class: "sort-th",   scope: "col",
                                    aria_sort: "{aria_sort_for(&sort.read(), SortColumn::TaxonName)}",
                                    onclick: toggle_sort(SortColumn::TaxonName),
                                    "Taxon "    span { class: "sort-icon", "aria-hidden": "true", {sort_icon(SortColumn::TaxonName)} }
                                }
                                th { class: "sort-th",   scope: "col",
                                    aria_sort: "{aria_sort_for(&sort.read(), SortColumn::RefTitle)}",
                                    onclick: toggle_sort(SortColumn::RefTitle),
                                    "Reference " span { class: "sort-icon", "aria-hidden": "true", {sort_icon(SortColumn::RefTitle)} }
                                }
                                th { class: "sort-th",   scope: "col",
                                    aria_sort: "{aria_sort_for(&sort.read(), SortColumn::PubYear)}",
                                    onclick: toggle_sort(SortColumn::PubYear),
                                    "Year "     span { class: "sort-icon", "aria-hidden": "true", {sort_icon(SortColumn::PubYear)} }
                                }
                            }
                        }
                        tbody {
                            for entry in page_rows.iter() {
                                Row { key: "{entry.compound_qid}-{entry.taxon_qid}-{entry.reference_qid}",
                                      entry: entry.clone() }
                            }
                        }
                    }
                }

                Pager { cur, total_pages, total,
                    on_prev: move |_| { let mut p = page.write(); if *p > 0 { *p -= 1; } },
                    on_next: move |_| { let mut p = page.write(); if *p + 1 < total_pages { *p += 1; } } }
            }
        }
    }
}

// ── Sub-components ────────────────────────────────────────────────────────────

#[component]
fn StatBadge(value: usize, label: &'static str) -> Element {
    rsx! {
        div { class: "stat-badge",
            span { class: "stat-value", "{value}" }
            span { class: "stat-label", "{label}" }
        }
    }
}

#[component]
fn Pager(
    cur: usize,
    total_pages: usize,
    total: usize,
    on_prev: EventHandler<MouseEvent>,
    on_next: EventHandler<MouseEvent>,
) -> Element {
    let start = cur * PAGE_SIZE + 1;
    let end   = ((cur + 1) * PAGE_SIZE).min(total);
    rsx! {
        div { class: "pagination-bar",
            button { class: "btn btn-sm", disabled: cur == 0,
                onclick: move |e| on_prev.call(e), "← Prev" }
            span { class: "page-info",
                "{start}–{end} of {total}  (page {cur+1} / {total_pages})"
            }
            button { class: "btn btn-sm", disabled: cur + 1 >= total_pages,
                onclick: move |e| on_next.call(e), "Next →" }
        }
    }
}

#[component]
fn Row(entry: CompoundEntry) -> Element {
    let wu  = entry.compound_url();
    let tu  = entry.taxon_url();
    let ru  = entry.reference_url();
    let su  = entry.scholia_url();
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
                    a { href: "{url}", target: "_blank", rel: "noopener noreferrer",
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
                    a { href: "{wu}", target: "_blank", rel: "noopener noreferrer",
                        class: "primary-link", "{name}" }
                }
                div { class: "badge-row",
                    a { href: "{wu}", target: "_blank", rel: "noopener noreferrer",
                        class: "id-badge wd", title: "Open in Wikidata",
                        aria_label: "Wikidata {entry.compound_qid}",
                        "{entry.compound_qid}" }
                    a { href: "{su}", target: "_blank", rel: "noopener noreferrer",
                        class: "id-badge sc", title: "Open in Scholia", "Scholia" }
                    if let (Some(ik), Some(search_url)) = (&entry.inchikey, inchikey_search.as_deref()) {
                        a { href: "{search_url}", target: "_blank", rel: "noopener noreferrer",
                            class: "id-badge mono", title: "{ik}",
                            aria_label: "Search Wikidata for InChIKey {ik}",
                            "{short_inchikey(ik)}"
                        }
                    }
                }
            }

            // ── Mass ────────────────────────────────────────────────────────
            td { class: "td-num",
                if let Some(m) = entry.mass { span { "{m:.4}" } }
                else { span { class: "na", "—" } }
            }
            // ── Formula ─────────────────────────────────────────────────────
            td { class: "td-formula",
                if let Some(f) = &entry.formula { span { class: "formula", "{f}" } }
                else { span { class: "na", "—" } }
            }

            // ── Taxon: italic name link + QID badge ────────────────────────
            td { class: "td-taxon",
                div { class: "cell-primary",
                    a { href: "{tu}", target: "_blank", rel: "noopener noreferrer",
                        class: "primary-link taxon",
                        "{entry.taxon_name}" }
                }
                div { class: "badge-row",
                    a { href: "{tu}", target: "_blank", rel: "noopener noreferrer",
                        class: "id-badge wd", title: "Open in Wikidata",
                        aria_label: "Wikidata {entry.taxon_qid}",
                        "{entry.taxon_qid}" }
                }
            }

            // ── Reference: title link + QID / DOI / statement badges ────────
            td { class: "td-ref",
                div { class: "cell-primary",
                    if let Some(t) = &entry.ref_title {
                        a { href: "{ru}", target: "_blank", rel: "noopener noreferrer",
                            class: "primary-link", title: "{t}", "{truncate_title(t, 60)}" }
                    } else {
                        a { href: "{ru}", target: "_blank", rel: "noopener noreferrer",
                            class: "primary-link", "{entry.reference_qid}" }
                    }
                }
                div { class: "badge-row",
                    a { href: "{ru}", target: "_blank", rel: "noopener noreferrer",
                        class: "id-badge wd", title: "Open in Wikidata",
                        aria_label: "Wikidata {entry.reference_qid}",
                        "{entry.reference_qid}" }
                    if let Some(url) = doi {
                        a { href: "{url}", target: "_blank", rel: "noopener noreferrer",
                            class: "id-badge doi", title: "Open DOI", "DOI" }
                    }
                    if let (Some(url), Some(stmt)) = (statement_url.as_deref(), statement_id.as_deref()) {
                        a { href: "{url}", target: "_blank", rel: "noopener noreferrer",
                            class: "id-badge stmt mono", title: "{stmt}",
                            aria_label: "Wikidata statement {stmt}", "stmt" }
                    }
                }
            }

            // ── Year ────────────────────────────────────────────────────────
            td { class: "td-year",
                if let Some(y) = entry.pub_year { span { "{y}" } }
                else { span { class: "na", "—" } }
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
            if i > 0 { out.push(','); }
            out.push_str(&csv_escape(f));
        }
        out.push('\n');
    }
    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

