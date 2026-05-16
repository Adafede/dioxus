// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Pure URL codec for explore state.
//!
//! This module intentionally contains no browser/runtime side effects so it can
//! be tested on any target and reused by both startup parsing and URL builders.

use crate::app::view::AppView;
use crate::download::DownloadFormat;
use crate::i18n::Locale;
use crate::models::{ElementState, SearchCriteria, SmilesSearchType};
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InitialDownloadState {
    pub pending_format: Option<DownloadFormat>,
    pub pending_invalid_format: Option<String>,
    pub direct_execute: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InitialUrlState {
    pub criteria: SearchCriteria,
    pub view: AppView,
    pub locale: Locale,
    pub download: InitialDownloadState,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct CriteriaQueryDto {
    taxon: Option<String>,
    structure: Option<String>,
    structure_search_type: Option<SmilesSearchType>,
    smiles_threshold: Option<f64>,
    mass_filter: Option<RangeF64Dto>,
    year_filter: Option<RangeU16Dto>,
    formula_filter: Option<FormulaQueryDto>,
    has_explicit_taxon: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct FormulaQueryDto {
    exact: Option<String>,
    c_min: Option<u16>,
    c_max: Option<u16>,
    h_min: Option<u16>,
    h_max: Option<u16>,
    n_min: Option<u16>,
    n_max: Option<u16>,
    o_min: Option<u16>,
    o_max: Option<u16>,
    p_min: Option<u16>,
    p_max: Option<u16>,
    s_min: Option<u16>,
    s_max: Option<u16>,
    f_state: Option<ElementState>,
    cl_state: Option<ElementState>,
    br_state: Option<ElementState>,
    i_state: Option<ElementState>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RangeF64Dto {
    min: Option<f64>,
    max: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RangeU16Dto {
    min: Option<u16>,
    max: Option<u16>,
}

impl CriteriaQueryDto {
    fn parse(params: &BTreeMap<String, String>) -> Self {
        let parse_f64 = |name: &str| params.get(name).and_then(|v| v.parse::<f64>().ok());
        let parse_u16 = |name: &str| params.get(name).and_then(|v| v.parse::<u16>().ok());
        let formula_filter = params
            .get("formula_filter")
            .is_some_and(|v| is_true_flag(v))
            .then(|| FormulaQueryDto {
                exact: params.get("formula_exact").cloned(),
                c_min: parse_u16("c_min"),
                c_max: parse_u16("c_max"),
                h_min: parse_u16("h_min"),
                h_max: parse_u16("h_max"),
                n_min: parse_u16("n_min"),
                n_max: parse_u16("n_max"),
                o_min: parse_u16("o_min"),
                o_max: parse_u16("o_max"),
                p_min: parse_u16("p_min"),
                p_max: parse_u16("p_max"),
                s_min: parse_u16("s_min"),
                s_max: parse_u16("s_max"),
                f_state: params
                    .get("f_state")
                    .map(|v| ElementState::from_str(v).unwrap_or_default()),
                cl_state: params
                    .get("cl_state")
                    .map(|v| ElementState::from_str(v).unwrap_or_default()),
                br_state: params
                    .get("br_state")
                    .map(|v| ElementState::from_str(v).unwrap_or_default()),
                i_state: params
                    .get("i_state")
                    .map(|v| ElementState::from_str(v).unwrap_or_default()),
            });

        Self {
            taxon: params.get("taxon").cloned(),
            structure: params
                .get("structure")
                .cloned()
                .or_else(|| params.get("smiles").cloned()),
            structure_search_type: params
                .get("structure_search_type")
                .cloned()
                .or_else(|| params.get("smiles_search_type").cloned())
                .map(|v| {
                    if v == "similarity" {
                        SmilesSearchType::Similarity
                    } else {
                        SmilesSearchType::Substructure
                    }
                }),
            smiles_threshold: params
                .get("smiles_threshold")
                .and_then(|v| v.parse::<f64>().ok())
                .map(|v| v.clamp(0.05, 1.0)),
            mass_filter: params
                .get("mass_filter")
                .is_some_and(|v| is_true_flag(v))
                .then(|| RangeF64Dto {
                    min: parse_f64("mass_min"),
                    max: parse_f64("mass_max"),
                }),
            year_filter: params
                .get("year_filter")
                .is_some_and(|v| is_true_flag(v))
                .then(|| RangeU16Dto {
                    min: parse_u16("year_start"),
                    max: parse_u16("year_end"),
                }),
            formula_filter,
            has_explicit_taxon: params.contains_key("taxon"),
        }
    }

    fn into_criteria(self) -> SearchCriteria {
        let mut criteria = SearchCriteria::default();
        if let Some(taxon) = self.taxon {
            criteria.taxon = taxon;
        }
        if let Some(structure) = self.structure {
            criteria.smiles = structure;
        }
        if let Some(search_type) = self.structure_search_type {
            criteria.smiles_search_type = search_type;
        }
        if let Some(threshold) = self.smiles_threshold {
            criteria.smiles_threshold = threshold;
        }
        if let Some(range) = self.mass_filter {
            if let Some(v) = range.min {
                criteria.mass_min = v;
            }
            if let Some(v) = range.max {
                criteria.mass_max = v;
            }
        }
        if let Some(range) = self.year_filter {
            if let Some(v) = range.min {
                criteria.year_min = v;
            }
            if let Some(v) = range.max {
                criteria.year_max = v;
            }
        }
        if let Some(formula) = self.formula_filter {
            criteria.formula_enabled = true;
            if let Some(v) = formula.exact {
                criteria.formula_exact = v;
            }
            if let Some(v) = formula.c_min {
                criteria.c_min = v;
            }
            if let Some(v) = formula.c_max {
                criteria.c_max = v;
            }
            if let Some(v) = formula.h_min {
                criteria.h_min = v;
            }
            if let Some(v) = formula.h_max {
                criteria.h_max = v;
            }
            if let Some(v) = formula.n_min {
                criteria.n_min = v;
            }
            if let Some(v) = formula.n_max {
                criteria.n_max = v;
            }
            if let Some(v) = formula.o_min {
                criteria.o_min = v;
            }
            if let Some(v) = formula.o_max {
                criteria.o_max = v;
            }
            if let Some(v) = formula.p_min {
                criteria.p_min = v;
            }
            if let Some(v) = formula.p_max {
                criteria.p_max = v;
            }
            if let Some(v) = formula.s_min {
                criteria.s_min = v;
            }
            if let Some(v) = formula.s_max {
                criteria.s_max = v;
            }
            if let Some(v) = formula.f_state {
                criteria.f_state = v;
            }
            if let Some(v) = formula.cl_state {
                criteria.cl_state = v;
            }
            if let Some(v) = formula.br_state {
                criteria.br_state = v;
            }
            if let Some(v) = formula.i_state {
                criteria.i_state = v;
            }
        }

        if criteria.smiles.trim().is_empty() || self.has_explicit_taxon {
            return criteria;
        }

        criteria.taxon.clear();
        criteria
    }
}

pub fn parse_criteria_from_params(params: &BTreeMap<String, String>) -> SearchCriteria {
    CriteriaQueryDto::parse(params).into_criteria()
}

pub fn parse_startup_action_from_params(params: &BTreeMap<String, String>) -> InitialDownloadState {
    let wants_download = params.get("download").is_some_and(|v| is_true_flag(v));
    if !wants_download {
        let wants_execute = params.get("execute").is_some_and(|v| is_true_flag(v));
        return InitialDownloadState {
            direct_execute: wants_execute,
            ..InitialDownloadState::default()
        };
    }

    let requested = params
        .get("format")
        .map(|v| v.to_ascii_lowercase())
        .unwrap_or_else(|| "csv".to_string());
    let pending_format = DownloadFormat::from_str(&requested);

    InitialDownloadState {
        pending_format,
        pending_invalid_format: pending_format.is_none().then_some(requested),
        direct_execute: false,
    }
}

pub fn build_shareable_url(criteria: &SearchCriteria) -> Option<String> {
    let params = criteria.shareable_query_params();
    if params.is_empty() {
        return None;
    }
    let query = build_query_string_from_pairs(params.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    Some(format!("?{query}"))
}

/// Test whether a URL query-parameter value represents a boolean true flag.
///
/// Accepts `"1"`, `"true"`, `"yes"`, and `"on"` (case-insensitive, trimmed).
/// All other values — including absent keys — are treated as false.
pub(crate) fn is_true_flag(v: &str) -> bool {
    matches!(
        v.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(target_arch = "wasm32")]
pub fn build_query_string(params: &BTreeMap<String, String>) -> String {
    build_query_string_from_pairs(params.iter().map(|(k, v)| (k.as_str(), v.as_str())))
}

fn build_query_string_from_pairs<'a>(iter: impl Iterator<Item = (&'a str, &'a str)>) -> String {
    iter.map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_criteria_supports_formula_and_halogens() {
        let mut params = BTreeMap::new();
        params.insert("taxon".into(), "*".into());
        params.insert("formula_filter".into(), "true".into());
        params.insert("c_min".into(), "15".into());
        params.insert("c_max".into(), "25".into());
        params.insert("o_min".into(), "2".into());
        params.insert("o_max".into(), "8".into());
        params.insert("f_state".into(), "required".into());
        params.insert("cl_state".into(), "required".into());
        params.insert("br_state".into(), "excluded".into());
        params.insert("i_state".into(), "excluded".into());

        let crit = parse_criteria_from_params(&params);
        assert!(crit.formula_enabled);
        assert_eq!(crit.c_min, 15);
        assert_eq!(crit.c_max, 25);
        assert_eq!(crit.o_min, 2);
        assert_eq!(crit.o_max, 8);
        assert_eq!(crit.f_state, ElementState::Required);
        assert_eq!(crit.cl_state, ElementState::Required);
        assert_eq!(crit.br_state, ElementState::Excluded);
        assert_eq!(crit.i_state, ElementState::Excluded);
    }

    #[test]
    fn startup_action_execute_only() {
        let mut params = BTreeMap::new();
        params.insert("execute".into(), "true".into());
        let startup = parse_startup_action_from_params(&params);
        assert!(startup.pending_format.is_none());
        assert!(startup.pending_invalid_format.is_none());
        assert!(startup.direct_execute);
    }

    #[test]
    fn share_params_roundtrip_for_advanced_filters() {
        let mut crit = SearchCriteria {
            taxon: "*".into(),
            ..SearchCriteria::default()
        };
        crit.formula_enabled = true;
        crit.c_min = 15;
        crit.c_max = 25;
        crit.o_min = 2;
        crit.o_max = 8;
        crit.f_state = ElementState::Required;
        crit.cl_state = ElementState::Required;
        crit.br_state = ElementState::Excluded;
        crit.i_state = ElementState::Excluded;

        let params: BTreeMap<String, String> = crit.shareable_query_params().into_iter().collect();
        let reparsed = parse_criteria_from_params(&params);
        assert_eq!(reparsed.taxon, crit.taxon);
        assert_eq!(reparsed.c_min, crit.c_min);
        assert_eq!(reparsed.c_max, crit.c_max);
        assert_eq!(reparsed.o_min, crit.o_min);
        assert_eq!(reparsed.o_max, crit.o_max);
        assert_eq!(reparsed.f_state, crit.f_state);
        assert_eq!(reparsed.cl_state, crit.cl_state);
        assert_eq!(reparsed.br_state, crit.br_state);
        assert_eq!(reparsed.i_state, crit.i_state);
    }

    #[test]
    fn share_params_keep_formula_toggle_but_omit_default_formula_bounds() {
        let crit = SearchCriteria {
            taxon: "Fungi".into(),
            formula_enabled: true,
            ..SearchCriteria::default()
        };

        let params: BTreeMap<String, String> = crit.shareable_query_params().into_iter().collect();
        let reparsed = parse_criteria_from_params(&params);

        assert_eq!(params.get("taxon").map(String::as_str), Some("Fungi"));
        assert_eq!(
            params.get("formula_filter").map(String::as_str),
            Some("true")
        );
        assert!(!params.contains_key("c_min"));
        assert!(!params.contains_key("c_max"));
        assert!(!params.contains_key("cl_state"));
        assert!(reparsed.formula_enabled);
        assert_eq!(reparsed.c_min, SearchCriteria::default().c_min);
        assert_eq!(reparsed.c_max, SearchCriteria::default().c_max);
        assert_eq!(reparsed.cl_state, SearchCriteria::default().cl_state);
    }

    #[test]
    fn startup_action_download_has_priority_over_execute() {
        let mut params = BTreeMap::new();
        params.insert("download".into(), "yes".into());
        params.insert("execute".into(), "true".into());
        params.insert("format".into(), "rdf".into());
        let startup = parse_startup_action_from_params(&params);
        assert_eq!(startup.pending_format, Some(DownloadFormat::Rdf));
        assert!(startup.pending_invalid_format.is_none());
        assert!(!startup.direct_execute);
    }

    #[test]
    fn startup_action_invalid_download_format_is_preserved() {
        let mut params = BTreeMap::new();
        params.insert("download".into(), "1".into());
        params.insert("format".into(), "ttl".into());

        let startup = parse_startup_action_from_params(&params);
        assert!(startup.pending_format.is_none());
        assert_eq!(startup.pending_invalid_format.as_deref(), Some("ttl"));
        assert!(!startup.direct_execute);
    }
}
