// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::errors::ApiError;
use crate::state::{AppState, taxon_cache_get, taxon_cache_put};
use crate::types::{ExportArchiveFormat, SearchRequest};
use flate2::{Compression, write::GzEncoder};
use shared::lotus::models::{SearchCriteria, TaxonMatch};
use shared::lotus::{queries, sparql};

pub(crate) fn apply_request(req: &SearchRequest) -> Result<SearchCriteria, ApiError> {
    let mut c = SearchCriteria {
        taxon: req.taxon.clone().unwrap_or_default(),
        smiles: req.smiles.clone().unwrap_or_default(),
        ..Default::default()
    };

    if let Some(v) = req.smiles_search_type {
        c.smiles_search_type = v.into();
    }
    if let Some(v) = req.smiles_threshold {
        c.smiles_threshold = v.clamp(0.05, 1.0);
    }
    if let Some(v) = req.mass_min {
        c.mass_min = v.max(0.0);
    }
    if let Some(v) = req.mass_max {
        c.mass_max = v.max(c.mass_min);
    }
    if let Some(v) = req.year_min {
        c.year_min = v;
    }
    if let Some(v) = req.year_max {
        c.year_max = v.max(c.year_min);
    }

    let has_formula_input = req
        .formula_exact
        .as_deref()
        .is_some_and(|v| !v.trim().is_empty())
        || req.c_min.is_some()
        || req.c_max.is_some()
        || req.h_min.is_some()
        || req.h_max.is_some()
        || req.n_min.is_some()
        || req.n_max.is_some()
        || req.o_min.is_some()
        || req.o_max.is_some()
        || req.p_min.is_some()
        || req.p_max.is_some()
        || req.s_min.is_some()
        || req.s_max.is_some()
        || req.f_state.is_some()
        || req.cl_state.is_some()
        || req.br_state.is_some()
        || req.i_state.is_some();

    c.formula_enabled = has_formula_input;
    if let Some(v) = req.formula_exact.as_deref() {
        c.formula_exact = v.trim().to_string();
    }

    c.c_min = req.c_min.unwrap_or(c.c_min);
    c.c_max = req.c_max.unwrap_or(c.c_max);
    c.h_min = req.h_min.unwrap_or(c.h_min);
    c.h_max = req.h_max.unwrap_or(c.h_max);
    c.n_min = req.n_min.unwrap_or(c.n_min);
    c.n_max = req.n_max.unwrap_or(c.n_max);
    c.o_min = req.o_min.unwrap_or(c.o_min);
    c.o_max = req.o_max.unwrap_or(c.o_max);
    c.p_min = req.p_min.unwrap_or(c.p_min);
    c.p_max = req.p_max.unwrap_or(c.p_max);
    c.s_min = req.s_min.unwrap_or(c.s_min);
    c.s_max = req.s_max.unwrap_or(c.s_max);

    if c.c_min > c.c_max
        || c.h_min > c.h_max
        || c.n_min > c.n_max
        || c.o_min > c.o_max
        || c.p_min > c.p_max
        || c.s_min > c.s_max
    {
        return Err(ApiError::bad_request("Element min must be <= max"));
    }

    if let Some(v) = req.f_state {
        c.f_state = v.into();
    }
    if let Some(v) = req.cl_state {
        c.cl_state = v.into();
    }
    if let Some(v) = req.br_state {
        c.br_state = v.into();
    }
    if let Some(v) = req.i_state {
        c.i_state = v.into();
    }

    Ok(c)
}

pub(crate) fn build_execution_query(
    criteria: &SearchCriteria,
    resolved_taxon_qid: Option<&str>,
) -> String {
    let smiles = normalized_structure_input(&criteria.smiles);
    let base_query = if !smiles.is_empty() {
        let taxon_for_sachem = match resolved_taxon_qid {
            Some("*") => Some("Q2382443"),
            Some(qid) => Some(qid),
            None => None,
        };
        queries::query_sachem(
            &smiles,
            criteria.smiles_search_type,
            criteria.smiles_threshold,
            taxon_for_sachem,
        )
    } else {
        match resolved_taxon_qid {
            Some("*") | None => queries::query_all_compounds(),
            Some(qid) => queries::query_compounds_by_taxon(qid),
        }
    };

    queries::query_with_server_filters(&base_query, criteria)
}

pub(crate) fn normalized_structure_input(value: &str) -> String {
    let normalized = value.replace("\r\n", "\n").replace('\r', "\n");
    match queries::classify_structure(&normalized) {
        queries::StructureKind::MolfileV2000 | queries::StructureKind::MolfileV3000 => normalized,
        _ => normalized.trim().to_string(),
    }
}

pub(crate) async fn resolve_taxon_qid_cached(
    state: &AppState,
    taxon_input: String,
) -> Result<(Option<String>, Option<String>), ApiError> {
    let key = taxon_input.trim().to_lowercase();
    if !key.is_empty()
        && let Some(cached) = taxon_cache_get(state, &key)
    {
        return Ok(cached);
    }

    let resolved = resolve_taxon_qid(taxon_input).await?;
    if !key.is_empty() {
        taxon_cache_put(state, key, resolved.clone());
    }
    Ok(resolved)
}

async fn resolve_taxon_qid(
    taxon_input: String,
) -> Result<(Option<String>, Option<String>), ApiError> {
    let taxon = taxon_input.trim().to_string();
    if taxon.is_empty() {
        return Ok((None, None));
    }
    if taxon == "*" {
        return Ok((Some("*".to_string()), None));
    }
    if is_qid(&taxon) {
        return Ok((Some(taxon.to_ascii_uppercase()), None));
    }

    let sanitized = sanitize_taxon_input(&taxon);
    let query = queries::query_taxon_search(&sanitized);
    let csv = sparql::execute_sparql_bytes(&query)
        .await
        .map_err(|e| ApiError::upstream(format!("taxon lookup failed: {e}")))?;
    let matches = sparql::parse_taxon_csv_bytes(&csv)
        .map_err(|e| ApiError::upstream(format!("taxon parse failed: {e}")))?;

    if matches.is_empty() {
        return Err(ApiError::bad_request(format!("Taxon not found: {taxon}")));
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
        .ok_or_else(|| ApiError::bad_request("Could not resolve taxon"))?;

    let warning = if sanitized != taxon {
        Some(format!(
            "Taxon normalized from '{taxon}' to '{}' ({}).",
            best.name, best.qid
        ))
    } else if exact.len() > 1 || (exact.is_empty() && matches.len() > 1) {
        Some(format!(
            "Ambiguous taxon input. Using '{}' ({})",
            best.name, best.qid
        ))
    } else {
        None
    };

    Ok((Some(best.qid.clone()), warning))
}

fn sanitize_taxon_input(taxon: &str) -> String {
    let replaced = taxon.replace('_', " ");
    let mut parts = replaced.split_whitespace();
    let Some(first_word) = parts.next() else {
        return replaced;
    };
    let mut chars = first_word.chars();
    let mut out = match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    };
    for part in parts {
        out.push(' ');
        out.push_str(part);
    }
    out
}

fn is_qid(value: &str) -> bool {
    let v = value.trim();
    if v.len() < 2 {
        return false;
    }
    let mut chars = v.chars();
    let first = chars.next().unwrap_or_default();
    (first == 'Q' || first == 'q') && chars.all(|c| c.is_ascii_digit())
}

pub(crate) fn qlever_export_url(query: &str, action: &str) -> String {
    format!(
        "{}?query={}&action={action}",
        shared::sparql::QLEVER_WIKIDATA,
        urlencoding::encode(query)
    )
}

pub(crate) fn api_export_file_url(cache_key: &str, format: ExportArchiveFormat) -> String {
    format!("/v1/export-file/{cache_key}/{}", format.extension())
}

pub(crate) fn build_upstream_export_url(query: &str, format: ExportArchiveFormat) -> String {
    match format {
        ExportArchiveFormat::Csv => qlever_export_url(query, "csv_export"),
        ExportArchiveFormat::Json => qlever_export_url(query, "qlever_json_export"),
        ExportArchiveFormat::Rdf => qlever_export_url(
            &queries::query_construct_from_select(query),
            "turtle_export",
        ),
    }
}

pub(crate) fn gzip_bytes(input: &[u8]) -> std::io::Result<Vec<u8>> {
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(input)?;
    encoder.finish()
}

pub(crate) fn sanitize_download_filename(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.trim().chars() {
        if c.is_control() {
            continue;
        }
        match c {
            '/' | '\\' | '"' | '\'' | '\n' | '\r' => out.push('_'),
            _ => out.push(c),
        }
    }
    out.trim_matches('.').trim().to_string()
}
