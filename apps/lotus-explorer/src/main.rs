#![allow(non_snake_case)]

mod components;
mod export;
mod models;
mod queries;
mod sparql;

use components::results_table::ResultsTable;
use components::search_panel::{KetcherPanel, SearchPanel};
use dioxus::prelude::*;
use models::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

fn main() {
    console_log::init_with_level(log::Level::Debug).ok();
    dioxus::launch(App);
}

// ── Root component ────────────────────────────────────────────────────────────

#[component]
fn App() -> Element {
    let criteria: Signal<SearchCriteria> = use_signal(initial_criteria_from_url);
    let mut entries: Signal<Vec<CompoundEntry>> = use_signal(Vec::new);
    let mut loading: Signal<bool> = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut searched_once: Signal<bool> = use_signal(|| false);
    let mut taxon_notice: Signal<Option<String>> = use_signal(|| None);
    let mut resolved_qid: Signal<Option<String>> = use_signal(|| None);
    let mut query_hash: Signal<Option<String>> = use_signal(|| None);
    let mut result_hash: Signal<Option<String>> = use_signal(|| None);
    let mut shareable_url: Signal<Option<String>> = use_signal(|| None);
    let mut sparql_query: Signal<Option<String>> = use_signal(|| None);
    let mut truncated: Signal<Option<(usize, usize)>> = use_signal(|| None);
    let mut metadata_json: Signal<Option<String>> = use_signal(|| None);
    let mut export_rows: Signal<Vec<CompoundEntry>> = use_signal(Vec::new);
    let sort: Signal<SortState> = use_signal(SortState::default);
    let mut page: Signal<usize> = use_signal(|| 0usize);

    // ── Search handler ────────────────────────────────────────────────────────
    let on_search = move |_| {
        if *loading.read() {
            return;
        }
        let crit = criteria.read().clone();

        if !crit.is_valid() {
            *error.write() = Some("Please enter a taxon name / QID, or a SMILES structure.".into());
            return;
        }

        *error.write() = None;
        *searched_once.write() = true;
        *loading.write() = true;
        *entries.write() = Vec::new();
        *export_rows.write() = Vec::new();
        *taxon_notice.write() = None;
        *resolved_qid.write() = None;
        *query_hash.write() = None;
        *result_hash.write() = None;
        *shareable_url.write() = None;
        *sparql_query.write() = None;
        *truncated.write() = None;
        *metadata_json.write() = None;
        *page.write() = 0;

        spawn(async move {
            match do_search(crit.clone()).await {
                Ok(mut outcome) => {
                    // Client-side post-filters (mass, year) — mirrors Python FilterService
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

                    // Apply the display-only row cap *after* filtering so the
                    // hashes we compute below cover the whole filtered result
                    // (matches Python behaviour where the query itself has no
                    // LIMIT and the cap is purely a UI concern).
                    let full_count = outcome.rows.len();
                    let cap = TABLE_ROW_LIMIT;
                    let was_truncated = full_count > cap;
                    let (q_hash, r_hash) =
                        compute_hashes(outcome.qid.as_deref().unwrap_or(""), &crit, &outcome.rows);
                    let full_stats = DatasetStats::from_entries(&outcome.rows);
                    let meta_str = export::build_metadata_json(export::MetadataInputs {
                        criteria: &crit,
                        qid: outcome.qid.as_deref(),
                        stats: &full_stats,
                        query_hash: &q_hash,
                        result_hash: &r_hash,
                    });
                    // Keep a complete copy of the filtered rows so exports can
                    // cover the whole dataset even when the display is capped.
                    *export_rows.write() = outcome.rows.clone();
                    if was_truncated {
                        outcome.rows.truncate(cap);
                    }
                    let share_url = build_shareable_url(&crit);

                    *resolved_qid.write() = outcome.qid;
                    *taxon_notice.write() = outcome.warning;
                    *query_hash.write() = Some(q_hash);
                    *result_hash.write() = Some(r_hash);
                    *shareable_url.write() = share_url;
                    *sparql_query.write() = Some(outcome.query);
                    *metadata_json.write() = Some(meta_str);
                    *truncated.write() = if was_truncated {
                        Some((cap, full_count))
                    } else {
                        None
                    };
                    *entries.write() = outcome.rows;
                    *loading.write() = false;
                }
                Err(e) => {
                    *error.write() = Some(e);
                    *loading.write() = false;
                }
            }
        });
    };

    let stats = DatasetStats::from_entries(&entries.read());

    rsx! {
        style { dangerous_inner_html: shared::theme::BASE_CSS }
        style { dangerous_inner_html: APP_CSS }

        div { class: "app-layout",
            // ── Left sidebar ──────────────────────────────────────────────
            aside { class: "sidebar",
                SearchPanel { criteria, on_search, loading: *loading.read() }
            }

            // ── Main panel ────────────────────────────────────────────────
            main { class: "main-content",
                div { class: "page-header",
                    h1 { class: "page-title", "LOTUS Wikidata Explorer" }
                    p { class: "page-sub",
                        "Natural product occurrences — compound, taxon, reference."
                    }
                    if let Some(qid) = resolved_qid.read().as_deref() {
                        p { class: "page-meta",
                            span { class: "meta-key", "Resolved taxon" }
                            span { class: "meta-sep", ":" }
                            span { class: "meta-val mono", "{qid}" }
                        }
                    }
                    if let (Some(qh), Some(rh)) = (
                        query_hash.read().as_deref(),
                        result_hash.read().as_deref(),
                    )
                    {
                        p { class: "page-meta",
                            span { class: "meta-key", "Query hash" }
                            span { class: "meta-sep", ":" }
                            span { class: "meta-val mono", "{&qh[..12]}" }
                            span { class: "meta-sep", "·" }
                            span { class: "meta-key", "Result hash" }
                            span { class: "meta-sep", ":" }
                            span { class: "meta-val mono", "{&rh[..12]}" }
                        }
                    }
                }

                // Ketcher structure editor — full-width in the main panel so
                // the drawing canvas has enough room.
                KetcherPanel {}

                if let Some(share) = shareable_url.read().as_deref() {
                    div { class: "notice notice-info", role: "status",
                        span { class: "notice-label", "Share" }
                        code { class: "notice-value", "{share}" }
                    }
                }

                if let Some(warning) = taxon_notice.read().as_deref() {
                    div { class: "notice notice-warn", role: "status",
                        span { class: "notice-label", "Notice" }
                        span { class: "notice-value", "{warning}" }
                    }
                }

                if let Some((cap, total)) = *truncated.read() {
                    div { class: "notice notice-warn", role: "status",
                        span { class: "notice-label", "Truncated" }
                        span { class: "notice-value",
                            "Showing the first {cap} of {total} rows. Exports include the full filtered result set."
                        }
                    }
                }

                if let Some(msg) = error.read().as_deref() {
                    div { class: "notice notice-error", role: "alert",
                        span { class: "notice-label", "Error" }
                        span { class: "notice-value", "{msg}" }
                        button {
                            class: "notice-dismiss",
                            r#type: "button",
                            aria_label: "Dismiss error",
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
                        div { class: "spinner-lg", "aria-hidden": "true" }
                        p { "Querying Wikidata via QLever…" }
                        p { class: "loading-hint", "Large result sets may take several seconds." }
                    }
                } else if entries.read().is_empty() && error.read().is_none() && !*searched_once.read() {
                    WelcomeScreen {}
                } else {
                    ResultsTable {
                        entries: entries.read().clone(),
                        export_rows: export_rows.read().clone(),
                        stats,
                        sort,
                        page,
                        sparql_query: sparql_query.read().clone(),
                        metadata_json: metadata_json.read().clone(),
                        query_hash: query_hash.read().clone(),
                        result_hash: result_hash.read().clone(),
                        criteria: criteria.read().clone(),
                    }
                }

                Footer {}
            }
        }
    }
}

// ── Footer (same links as the Python notebook, cleaner markup) ───────────────

#[component]
fn Footer() -> Element {
    // Tiny helper to render a list of (href, text) pairs separated by `·`.
    let links = |items: &[(&'static str, &'static str)], class: &'static str| -> Vec<Element> {
        let mut out = Vec::with_capacity(items.len() * 2);
        for (i, (href, text)) in items.iter().enumerate() {
            if i > 0 {
                out.push(rsx! {
                    span { class: "footer-sep", "·" }
                });
            }
            out.push(rsx! {
                a {
                    class: "{class}",
                    href: "{href}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "{text}"
                }
            });
        }
        out
    };

    rsx! {
        footer { class: "app-footer",
            div { class: "footer-row",
                span { class: "footer-label", "Data" }
                {
                    links(
                            &[
                                ("https://www.wikidata.org/wiki/Q104225190", "LOTUS Initiative"),
                                ("https://www.wikidata.org/", "Wikidata"),
                            ],
                            "footer-link data",
                        )
                        .into_iter()
                }
            }
            div { class: "footer-row",
                span { class: "footer-label", "Code" }
                {
                    links(
                            &[
                                (
                                    "https://github.com/Adafede/dioxus/tree/main/apps/lotus-explorer",
                                    "lotus-explorer",
                                ),
                            ],
                            "footer-link code",
                        )
                        .into_iter()
                }
            }
            div { class: "footer-row",
                span { class: "footer-label", "Tools" }
                {
                    links(
                            &[
                                ("https://github.com/cdk/depict", "CDK Depict"),
                                ("https://idsm.elixir-czech.cz/", "IDSM"),
                                ("https://doi.org/10.1186/s13321-018-0282-y", "Sachem"),
                                ("https://qlever.dev/wikidata", "QLever"),
                            ],
                            "footer-link tool",
                        )
                        .into_iter()
                }
            }
            div { class: "footer-row",
                span { class: "footer-label", "License" }
                a {
                    class: "footer-link muted",
                    href: "https://creativecommons.org/publicdomain/zero/1.0/",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "CC0 1.0"
                }
                span { class: "footer-aside", " for data " }
                span { class: "footer-sep", "·" }
                a {
                    class: "footer-link muted",
                    href: "https://www.gnu.org/licenses/agpl-3.0.html",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "AGPL-3.0"
                }
                span { class: "footer-aside", " for code" }
            }
        }
    }
}

// ── Welcome screen ────────────────────────────────────────────────────────────

#[component]
fn WelcomeScreen() -> Element {
    rsx! {
        section { class: "welcome",
            div { class: "welcome-hero",
                h2 { "Browse natural product occurrences" }
                p { class: "welcome-lead",
                    "Every row ties a compound to the organism it has been reported from, "
                    "together with the primary literature reference. Data comes from the "
                    a {
                        href: "https://www.wikidata.org/wiki/Q104225190",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "LOTUS initiative"
                    }
                    ", stored on "
                    a {
                        href: "https://www.wikidata.org/",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "Wikidata"
                    }
                    " and queried via "
                    a {
                        href: "https://qlever.dev/wikidata",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "QLever"
                    }
                    "."
                }
            }

            div { class: "welcome-examples",
                h3 { "Try" }
                ul { class: "example-list",
                    ExRow {
                        value: "Gentiana lutea",
                        note: "Compounds from yellow gentian",
                    }
                    ExRow {
                        value: "Cannabis sativa",
                        note: "Compounds from Cannabis sativa and subtaxa",
                    }
                    ExRow {
                        value: "Q134630",
                        note: "Citrus genus — enter a bare Wikidata QID",
                    }
                    ExRow {
                        value: "*",
                        note: "All LOTUS compound–taxon–reference triples",
                    }
                    ExRow {
                        value: "c1ccccc1",
                        note: "Paste a SMILES in the structure box — no taxon required",
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
}

async fn do_search(crit: SearchCriteria) -> Result<SearchOutcome, String> {
    let taxon = crit.taxon.trim().to_string();
    // Preserve Molfile blocks verbatim — leading blank lines and whitespace
    // on header rows (lines 1–3 of a V2000/V3000 CTAB) are significant and
    // must reach SACHEM untouched, otherwise the query silently returns
    // no matches. Only trim single-line SMILES inputs. Mirrors the Python
    // `validate_and_escape` behaviour.
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
        let sanitized = sanitize_taxon_input(&taxon);
        let query = queries::query_taxon_search(&sanitized);
        let csv = sparql::execute_sparql(&query)
            .await
            .map_err(|e| format!("Taxon search failed: {e}"))?;
        let matches =
            sparql::parse_taxon_csv(&csv).map_err(|e| format!("Taxon parse failed: {e}"))?;
        if matches.is_empty() {
            return Err(format!("Taxon '{taxon}' not found in Wikidata."));
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
            warning = Some(format!(
                "Input standardized from '{taxon}' to '{sanitized}'."
            ));
        }
        if exact.len() > 1 || (exact.is_empty() && matches.len() > 1) {
            let names = matches
                .iter()
                .take(4)
                .map(|m| format!("{} ({})", m.name, m.qid))
                .collect::<Vec<_>>()
                .join(", ");
            warning = Some(format!(
                "Ambiguous taxon name; using {} ({}). Candidates: {}",
                best.name, best.qid, names
            ));
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
            Some("*") | None => queries::query_all_compounds(),
            Some(qid) => queries::query_compounds_by_taxon(qid),
        }
    };

    let csv = sparql::execute_sparql(&sparql_query)
        .await
        .map_err(|e| format!("Query failed: {e}"))?;
    let rows = sparql::parse_compounds_csv(&csv).map_err(|e| format!("Parse error: {e}"))?;

    Ok(SearchOutcome {
        rows,
        qid: taxon_qid,
        warning,
        query: sparql_query,
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
    let query_hash = format!("{:x}", Sha256::digest(query_source.as_bytes()));

    let mut compounds = rows
        .iter()
        .map(|e| e.compound_qid.as_str())
        .collect::<Vec<_>>();
    compounds.sort_unstable();
    compounds.dedup();
    let result_source = compounds.join("|");
    let result_hash = format!("{:x}", Sha256::digest(result_source.as_bytes()));

    (query_hash, result_hash)
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

fn initial_criteria_from_url() -> SearchCriteria {
    let mut criteria = SearchCriteria::default();
    let params = read_url_query_params();

    if let Some(taxon) = params.get("taxon") {
        criteria.taxon = taxon.clone();
    }
    if let Some(structure) = params
        .get("structure")
        .cloned()
        .or_else(|| params.get("smiles").cloned())
    {
        criteria.smiles = structure;
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

    criteria
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

// ── App-specific CSS (layout + LOTUS/Wikidata palette) ───────────────────────

const APP_CSS: &str = r#"
/* ── Wikidata colour palette (matches Python CONFIG) ─────────────────────── */
:root {
  --wd-compound:  #990000;   /* red   — compounds  */
  --wd-taxon:     #339966;   /* green — taxa       */
  --wd-reference: #006699;   /* blue  — references */
  --wd-hyperlink: #3377c4;
}

/* ── Layout ──────────────────────────────────────────────────────────────── */
.app-layout    { display:flex; height:100vh; overflow:hidden; }
.sidebar       { width:320px; min-width:280px; height:100vh; overflow-y:auto;
                 background:var(--bg2); border-right:1px solid var(--border); flex-shrink:0; }
.main-content  { flex:1; height:100vh; overflow-y:auto; display:flex; flex-direction:column; }

/* ── Page header ─────────────────────────────────────────────────────────── */
.page-header { padding:22px 28px 16px; border-bottom:1px solid var(--border); background:var(--bg2); }
.page-title  { font-size:18px; font-weight:600; letter-spacing:-.01em; }
.page-sub    { font-size:13px; color:var(--text2); margin-top:4px; }
.page-meta   { font-size:11px; color:var(--text3); margin-top:6px; display:flex; gap:6px;
               flex-wrap:wrap; align-items:baseline; }
.meta-key    { text-transform:uppercase; letter-spacing:.8px; font-weight:600; }
.meta-val.mono { font-family:var(--mono); color:var(--text2); }
.meta-sep    { color:var(--text3); }

/* ── Notices (info / warn / error) ───────────────────────────────────────── */
.notice           { margin:12px 28px 0; padding:10px 14px; display:flex; align-items:baseline;
                    gap:12px; border-radius:var(--radius); font-size:13px;
                    border:1px solid var(--border); background:var(--surface); }
.notice-label     { text-transform:uppercase; letter-spacing:.6px; font-size:10px; font-weight:600;
                    padding:2px 6px; border-radius:3px; flex-shrink:0; }
.notice-value     { flex:1; color:var(--text); word-break:break-word; }
.notice-info      { border-color:rgba(88,166,255,.3);  background:rgba(88,166,255,.05); }
.notice-info     .notice-label { background:rgba(88,166,255,.15); color:var(--accent); }
.notice-warn      { border-color:rgba(210,153,34,.35); background:rgba(210,153,34,.05); }
.notice-warn     .notice-label { background:rgba(210,153,34,.2);  color:var(--yellow); }
.notice-error     { border-color:rgba(248,81,73,.4);   background:rgba(248,81,73,.05); }
.notice-error    .notice-label { background:rgba(248,81,73,.2);   color:var(--red); }
.notice-dismiss   { margin-left:auto; background:none; border:0; color:inherit; cursor:pointer;
                    font-size:18px; line-height:1; padding:0 4px; opacity:.7; }
.notice-dismiss:hover { opacity:1; }

/* ── Search panel (sidebar) ──────────────────────────────────────────────── */
.search-panel    { padding:22px 20px; display:flex; flex-direction:column; gap:18px; }

.form-section    { display:flex; flex-direction:column; gap:5px; }
.form-section.nested { padding-left:10px; border-left:1px solid var(--border); margin-top:4px; }
.form-label      { font-size:11px; font-weight:600; color:var(--text);
                   text-transform:uppercase; letter-spacing:.6px; }
.form-label.sm   { font-size:10px; font-weight:500; color:var(--text2);
                   text-transform:none; letter-spacing:0; }
.form-hint       { font-size:11px; color:var(--text3); }
.radio-group     { display:flex; gap:14px; }
.radio-label     { display:flex; align-items:center; gap:6px; font-size:12px; cursor:pointer;
                   color:var(--text2); }
.radio-label input { accent-color:var(--accent); }
.range-input     { width:100%; accent-color:var(--accent); margin-top:4px; }
.range-inputs    { display:flex; align-items:flex-end; gap:8px; }
.range-pair      { display:flex; flex-direction:column; gap:3px; }
.range-sep       { color:var(--text3); padding-bottom:8px; }

/* ── Structure section: format pill + Ketcher editor ─────────────────────── */
.form-textarea.mono, .mono { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
.kind-pill        { display:inline-block; padding:1px 7px; border-radius:999px; font-size:10px;
                    font-weight:700; letter-spacing:.4px; text-transform:uppercase;
                    margin-right:6px; color:#fff; background:var(--text3); }
.kind-pill[data-kind="smiles"]  { background:var(--accent2, #5b6cff); }
.kind-pill[data-kind="mol2000"] { background:#c97a2b; }
.kind-pill[data-kind="mol3000"] { background:#2b8f57; }
.kind-note        { color:var(--text3); }

.ketcher-panel    { margin:14px 28px 0; border:1px solid var(--border);
                    border-radius:var(--radius); background:var(--bg2); }
.ketcher-panel > summary { cursor:pointer; padding:12px 16px; font-size:13px; font-weight:600;
                           color:var(--text); letter-spacing:.3px; user-select:none;
                           list-style:none; }
.ketcher-panel > summary::-webkit-details-marker { display:none; }
.ketcher-panel > summary::before { content:"▸ "; color:var(--text3); font-size:11px; }
.ketcher-panel[open] > summary::before { content:"▾ "; }
.ketcher-wrap     { padding:0 14px 14px; display:flex; flex-direction:column; gap:10px; }
.ketcher-iframe   { width:100%; height:min(78vh, 820px); min-height:600px;
                    border:1px solid var(--border); border-radius:var(--radius-sm);
                    background:#fff; }
.ketcher-hint     { margin-top:2px; font-size:12px; color:var(--text2); }
.ketcher-install  { color:var(--text3); font-size:10px; line-height:1.4; }
.ketcher-install a { color:var(--accent); }

/* Search button — subtle, not gradient. */
.search-btn      { display:flex; align-items:center; justify-content:center; gap:8px;
                   background:var(--accent2); color:#fff; border:0; border-radius:var(--radius-sm);
                   padding:10px 16px; font-size:13px; font-weight:600; cursor:pointer;
                   transition:background .15s; }
.search-btn:hover:not(:disabled)    { background:var(--accent); }
.search-btn:disabled                { opacity:.5; cursor:not-allowed; }

/* ── Welcome ─────────────────────────────────────────────────────────────── */
.welcome         { padding:40px 28px; max-width:720px; display:flex; flex-direction:column; gap:32px; }
.welcome-hero h2 { font-size:22px; font-weight:600; letter-spacing:-.01em; }
.welcome-lead    { font-size:14px; color:var(--text2); margin-top:10px; line-height:1.6; max-width:560px; }
.welcome-examples h3 { font-size:11px; font-weight:600; color:var(--text3);
                       text-transform:uppercase; letter-spacing:1.2px; margin-bottom:10px; }
.example-list    { list-style:none; display:flex; flex-direction:column; gap:6px; }
.example-item    { display:flex; gap:14px; align-items:baseline; padding:6px 0;
                   border-bottom:1px solid var(--border); }
.example-item:last-child { border-bottom:0; }
.example-value   { font-family:var(--mono); font-size:12px; color:var(--accent);
                   background:var(--surface); padding:2px 8px; border-radius:3px;
                   min-width:160px; white-space:nowrap; }
.example-note    { font-size:13px; color:var(--text2); }

/* ── Results ─────────────────────────────────────────────────────────────── */
.results-wrap    { padding:16px 28px; display:flex; flex-direction:column; gap:14px; }
.results-toolbar { display:flex; align-items:center; justify-content:space-between; gap:12px;
                   flex-wrap:wrap; }
.toolbar-actions { display:flex; gap:8px; align-items:center; }

/* SPARQL query panel */
.query-panel     { background:var(--surface); border:1px solid var(--border);
                   border-radius:var(--radius); }
.query-panel > summary { cursor:pointer; padding:8px 14px; font-size:11px; color:var(--text2);
                         user-select:none; text-transform:uppercase; letter-spacing:.8px;
                         font-weight:600; list-style:none; }
.query-panel > summary::-webkit-details-marker { display:none; }
.query-panel > summary::before { content:"▸ "; color:var(--text3); }
.query-panel[open] > summary::before { content:"▾ "; }
.query-panel > summary:hover { color:var(--text); }
.query-text      { padding:12px 16px; margin:0; font-family:var(--mono); font-size:11.5px;
                   color:var(--text); background:var(--bg); border-top:1px solid var(--border);
                   white-space:pre-wrap; word-break:break-word; max-height:320px; overflow:auto; }

.table-scroll    { overflow-x:auto; border:1px solid var(--border); border-radius:var(--radius); }
.results-table   { width:100%; border-collapse:collapse; font-size:13px; }
.results-table thead { position:sticky; top:0; z-index:2; background:var(--bg2); }
.sort-th, .th-static { padding:10px 12px; text-align:left; font-size:10px; font-weight:600;
                       color:var(--text3); border-bottom:1px solid var(--border);
                       white-space:nowrap; user-select:none;
                       text-transform:uppercase; letter-spacing:.8px; }
.sort-th         { cursor:pointer; }
.sort-th:hover   { color:var(--text); }
.sort-icon       { color:var(--text3); font-size:10px; margin-left:2px; }
.data-row        { border-bottom:1px solid var(--border); transition:background .1s; }
.data-row:hover  { background:var(--surface); }
.data-row td     { padding:10px 12px; vertical-align:middle; }

/* ── Stats (inline tags instead of emoji badges) ─────────────────────────── */
.stat-bar        { display:flex; flex-wrap:wrap; gap:18px; align-items:baseline; }
.stat-badge      { display:flex; flex-direction:column; gap:2px; }
.stat-icon       { display:none; }          /* keep markup minimal, hide legacy icons */
.stat-value      { font-size:17px; font-weight:600; color:var(--text); font-variant-numeric:tabular-nums; }
.stat-label      { font-size:10px; color:var(--text3); text-transform:uppercase; letter-spacing:.8px;
                   font-weight:600; }

/* ── Cell types ──────────────────────────────────────────────────────────── */
.td-depict       { width:130px; padding:6px 10px !important; }
.depict-img      { display:block; background:#fff; border:1px solid var(--border);
                   border-radius:4px; width:120px; height:72px; object-fit:contain; }
.td-compound     { min-width:220px; max-width:280px; }
.td-taxon        { min-width:170px; max-width:230px; }
.td-ref          { min-width:220px; max-width:320px; }
.cell-primary    { font-weight:500; }
.primary-link    { color:var(--text); }
.primary-link:hover { color:var(--wd-hyperlink); text-decoration:none; }
.primary-link.taxon { color:var(--wd-taxon); font-style:italic; }
.primary-link.taxon:hover { color:#2d8656; }

/* Unified ID badge */
.badge-row       { display:flex; flex-wrap:wrap; gap:4px; margin-top:4px; }
.id-badge        { display:inline-block; font-size:10px; padding:1px 6px; border-radius:3px;
                   font-weight:600; text-decoration:none !important; line-height:1.5;
                   border:1px solid transparent; font-family:var(--mono); }
.id-badge:hover  { filter:brightness(1.15); }
.td-compound .id-badge.wd { background:rgba(153,0,0,.12);  color:var(--wd-compound);
                            border-color:rgba(153,0,0,.35); }
.td-taxon    .id-badge.wd { background:rgba(51,153,102,.12); color:var(--wd-taxon);
                            border-color:rgba(51,153,102,.35); }
.td-ref      .id-badge.wd { background:rgba(0,102,153,.15); color:var(--wd-reference);
                            border-color:rgba(0,102,153,.35); }
.id-badge.sc     { background:rgba(88,166,255,.12); color:var(--accent); border-color:rgba(88,166,255,.3); }
.id-badge.doi    { background:rgba(210,153,34,.12); color:var(--yellow);  border-color:rgba(210,153,34,.35); }
.id-badge.stmt   { background:rgba(188,140,255,.1);  color:var(--purple);  border-color:rgba(188,140,255,.28); }
.id-badge.mono   { background:var(--surface); color:var(--text2); border-color:var(--border); }

.td-mono         { font-family:var(--mono); font-size:11px; white-space:nowrap; }
.td-num          { text-align:right; white-space:nowrap; font-variant-numeric:tabular-nums; }
.td-formula .formula { font-family:var(--mono); font-size:12px; color:var(--purple); }
.td-year         { text-align:center; color:var(--text2); white-space:nowrap; font-variant-numeric:tabular-nums; }
.na              { color:var(--text3); }

/* ── Footer ──────────────────────────────────────────────────────────────── */
.app-footer      { margin-top:auto; padding:20px 28px; border-top:1px solid var(--border);
                   background:var(--bg2); color:var(--text2);
                   display:flex; flex-direction:column; gap:8px; font-size:12px; }
.footer-row      { display:flex; flex-wrap:wrap; gap:6px; align-items:baseline; }
.footer-label    { color:var(--text3); font-weight:600; text-transform:uppercase;
                   font-size:10px; letter-spacing:1px; min-width:56px; }
.footer-sep      { color:var(--text3); }
.footer-aside    { color:var(--text3); font-size:11px; }
.footer-link           { color:var(--text);  text-decoration:none; }
.footer-link:hover     { text-decoration:underline; }
.footer-link.data      { color:var(--wd-compound); }
.footer-link.code      { color:var(--wd-taxon); }
.footer-link.tool      { color:var(--wd-reference); }
.footer-link.muted     { color:var(--text2); }

/* ── Download button group ───────────────────────────────────────────────── */
.dl-group        { display:inline-flex; isolation:isolate; }
.dl-group .btn   { border-radius:0; border-right-width:0; }
.dl-group .btn:hover { z-index:1; }
.dl-group .btn:first-child { border-top-left-radius:var(--radius-sm);
                             border-bottom-left-radius:var(--radius-sm); }
.dl-group .btn:last-child  { border-top-right-radius:var(--radius-sm);
                             border-bottom-right-radius:var(--radius-sm);
                             border-right-width:1px; }

/* ── Responsive ──────────────────────────────────────────────────────────── */
@media (max-width:768px) {
  .app-layout   { flex-direction:column; height:auto; min-height:100vh; overflow:visible; }
  .sidebar      { width:100%; height:auto; max-height:none; overflow-y:visible; }
  .main-content { height:auto; min-height:0; overflow-y:visible; }
  .page-header, .welcome, .results-wrap, .app-footer { padding-left:18px; padding-right:18px; }
  .notice       { margin-left:18px; margin-right:18px; }
  .ketcher-panel { margin-left:18px; margin-right:18px; }
  .ketcher-iframe { height:min(70vh, 560px); min-height:420px; }
}
"#;
