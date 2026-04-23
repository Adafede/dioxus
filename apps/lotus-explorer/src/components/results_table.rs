use crate::export;
use crate::i18n::{
    CountNoun, Locale, TextKey, aria_chemical_structure, aria_search_inchikey, aria_wikidata_entity,
    aria_wikidata_statement, count_label, t,
};
use crate::models::*;
use crate::queries;
use crate::sparql;
use dioxus::prelude::*;
use shared::sparql::SparqlResponseFormat;
use std::sync::Arc;

const TABLE_SCROLL_ID: &str = "results-table-scroll";
const VIRTUAL_OVERSCAN_ROWS: usize = 12;
const ROW_HEIGHT_PX_COMFORTABLE: usize = 114;
const TABLE_VIEWPORT_FALLBACK_PX: usize = 640;

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
    display_capped_rows: bool,
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
    // Exports are served from fresh endpoint requests (CSV/JSON/RDF), so
    // the table only keeps the preview rows needed for rendering.
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

    #[allow(unused_mut)]
    let mut scroll_top_px = use_signal(|| 0usize);
    #[allow(unused_mut)]
    let mut viewport_height_px = use_signal(|| TABLE_VIEWPORT_FALLBACK_PX);

    let row_height_px = ROW_HEIGHT_PX_COMFORTABLE;
    let window_rows = (((*viewport_height_px.read()).saturating_add(row_height_px - 1))
        / row_height_px)
        .max(1)
        .saturating_add(VIRTUAL_OVERSCAN_ROWS * 2);
    let first_visible_row = ((*scroll_top_px.read()) / row_height_px).min(total);
    let start_row = first_visible_row.saturating_sub(VIRTUAL_OVERSCAN_ROWS);
    let end_row = start_row.saturating_add(window_rows).min(total);
    let top_spacer_px = start_row.saturating_mul(row_height_px);
    let bottom_spacer_px = total.saturating_sub(end_row).saturating_mul(row_height_px);
    let visible_count = end_row.saturating_sub(start_row);

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

    // Metadata filename mirrors Python: `{query_hash}_{result_hash}_metadata.json`.
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
    let mut download_busy = use_signal(|| false);
    let mut download_status: Signal<Option<String>> = use_signal(|| None);
    let download_status_text = download_status
        .read()
        .clone()
        .unwrap_or_else(|| t(locale, TextKey::PreparingDownload).to_string());
    let _ = total_matches;

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
                            aria_label: "{t(locale, TextKey::DownloadResults)}",
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
                                            *download_status.write() = Some(
                                                t(locale, TextKey::StartingCsvDownload).to_string(),
                                            );
                                            let q = q.clone();
                                            spawn(async move {
                                                if let Ok(body) = sparql::execute_sparql(&q).await {
                                                    trigger_download(&filename, "text/csv;charset=utf-8", &body);
                                                }
                                                *download_busy.write() = false;
                                                *download_status.write() = None;
                                            });
                                        }
                                    },
                                    title: "{t(locale, TextKey::DownloadCsvTitle)}",
                                    aria_label: "{t(locale, TextKey::DownloadCsvTitle)}",
                                    "{t(locale, TextKey::DownloadCsvLabel)}"
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
                                            *download_status.write() = Some(
                                                t(locale, TextKey::PreparingJsonDownload).to_string(),
                                            );
                                            spawn(async move {
                                                if let Ok(body) = sparql::execute_sparql_format(
                                                        &q,
                                                        SparqlResponseFormat::SparqlJson,
                                                    )
                                                    .await
                                                {
                                                    trigger_download(
                                                        &filename,
                                                        "application/sparql-results+json;charset=utf-8",
                                                        &body,
                                                    );
                                                }
                                                *download_busy.write() = false;
                                                *download_status.write() = None;
                                            });
                                        }
                                    },
                                    title: "{t(locale, TextKey::DownloadJsonTitle)}",
                                    aria_label: "{t(locale, TextKey::DownloadJsonTitle)}",
                                    "{t(locale, TextKey::DownloadJsonLabel)}"
                                }
                                button {
                                    class: "btn btn-sm",
                                    r#type: "button",
                                    disabled: *download_busy.read(),
                                    onclick: {
                                        let q = query.to_string();
                                        move |_| {
                                            let q = queries::query_construct_from_select(&q);
                                            let filename = rdf_filename.read().clone();
                                            *download_busy.write() = true;
                                            *download_status.write() = Some(
                                                t(locale, TextKey::PreparingRdfDownload).to_string(),
                                            );
                                            spawn(async move {
                                                if let Ok(body) = sparql::execute_sparql_format(
                                                        &q,
                                                        SparqlResponseFormat::Turtle,
                                                    )
                                                    .await
                                                {
                                                    trigger_download(&filename, "text/turtle;charset=utf-8", &body);
                                                }
                                                *download_busy.write() = false;
                                                *download_status.write() = None;
                                            });
                                        }
                                    },
                                    title: "{t(locale, TextKey::DownloadRdfTitle)}",
                                    aria_label: "{t(locale, TextKey::DownloadRdfTitle)}",
                                    "{t(locale, TextKey::DownloadRdfLabel)}"
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
                                    title: "{t(locale, TextKey::DownloadMetadataTitle)}",
                                    aria_label: "{t(locale, TextKey::DownloadMetadataTitle)}",
                                    "{t(locale, TextKey::DownloadMetadataLabel)}"
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
                                            *scroll_top_px.write() = top;
                                            if height > 0 {
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
                                    aria_sort: "{aria_sort_for(&sort.read(), SortColumn::Name)}",
                                    onclick: toggle_sort(SortColumn::Name),
                                    "{t(locale, TextKey::Compound)} "
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
                                    "{t(locale, TextKey::Mass)} "
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
                                    "{t(locale, TextKey::Formula)} "
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
                                    "{t(locale, TextKey::TaxonCol)} "
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
                                    "{t(locale, TextKey::Reference)} "
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
                                    "{t(locale, TextKey::Year)} "
                                    span {
                                        class: "sort-icon",
                                        "aria-hidden": "true",
                                        {sort_icon(SortColumn::PubYear)}
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
                                let rows = entries.read();
                                let order = sorted_indices.read();
                                rsx! {
                                    for i in order.iter().skip(start_row).take(visible_count).copied() {
                                        Row { key: "{i}", locale, entry: rows[i as usize].clone() }
                                    }
                                }
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
fn Row(locale: Locale, entry: CompoundEntry) -> Element {
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
                        title: "{t(locale, TextKey::OpenInWikidata)}",
                        aria_label: "{aria_wikidata_entity(locale, &compound_qid)}",
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
                            aria_label: "{aria_search_inchikey(locale, ik)}",
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
                        title: "{t(locale, TextKey::OpenInWikidata)}",
                        aria_label: "{aria_wikidata_entity(locale, &taxon_qid)}",
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
                        title: "{t(locale, TextKey::OpenInWikidata)}",
                        aria_label: "{aria_wikidata_entity(locale, &reference_qid)}",
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
                    if let Some(stmt) = statement_id.as_deref() {
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
        use wasm_bindgen::JsCast;

        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };

        let parts = js_sys::Array::new();
        parts.push(&wasm_bindgen::JsValue::from_str(body));

        let blob = {
            let options = web_sys::BlobPropertyBag::new();
            options.set_type(mime);
            web_sys::Blob::new_with_str_sequence_and_options(&parts, &options)
                .or_else(|_| web_sys::Blob::new_with_str_sequence(&parts))
        };
        let Ok(blob) = blob else {
            return;
        };

        let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
            return;
        };

        let anchor = document
            .create_element("a")
            .ok()
            .and_then(|el| el.dyn_into::<web_sys::HtmlAnchorElement>().ok());

        if let (Some(a), Some(body_el)) = (anchor, document.body()) {
            a.set_href(&url);
            a.set_download(filename);
            a.set_rel("noopener noreferrer");
            let _ = body_el.append_child(&a);
            a.click();
            let _ = body_el.remove_child(&a);
        } else {
            let _ = window.open_with_url(&url);
        }

        let _ = web_sys::Url::revoke_object_url(&url);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (filename, mime, body);
    }
}
