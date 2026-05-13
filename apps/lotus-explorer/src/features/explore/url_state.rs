// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::app::view::AppView;
use crate::i18n::Locale;
use crate::models::{ElementState, SearchCriteria, SmilesSearchType};
use std::collections::BTreeMap;

pub fn initial_criteria_from_url() -> SearchCriteria {
    let params = read_url_query_params();
    parse_criteria_from_params(&params)
}

pub fn parse_criteria_from_params(params: &BTreeMap<String, String>) -> SearchCriteria {
    let mut criteria = SearchCriteria::default();
    let parse_f64 = |name: &str| params.get(name).and_then(|v| v.parse::<f64>().ok());
    let parse_u16 = |name: &str| params.get(name).and_then(|v| v.parse::<u16>().ok());
    let has_explicit_taxon = params.contains_key("taxon");
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
    if let Some(threshold) = params.get("smiles_threshold")
        && let Ok(v) = threshold.parse::<f64>()
    {
        criteria.smiles_threshold = v.clamp(0.05, 1.0);
    }

    if params
        .get("mass_filter")
        .map(|v| is_true_flag(v))
        .unwrap_or(false)
    {
        if let Some(v) = parse_f64("mass_min") {
            criteria.mass_min = v;
        }
        if let Some(v) = parse_f64("mass_max") {
            criteria.mass_max = v;
        }
    }

    if params
        .get("year_filter")
        .map(|v| is_true_flag(v))
        .unwrap_or(false)
    {
        if let Some(v) = parse_u16("year_start") {
            criteria.year_min = v;
        }
        if let Some(v) = parse_u16("year_end") {
            criteria.year_max = v;
        }
    }

    if params
        .get("formula_filter")
        .map(|v| is_true_flag(v))
        .unwrap_or(false)
    {
        criteria.formula_enabled = true;
        if let Some(v) = params.get("formula_exact") {
            criteria.formula_exact = v.clone();
        }
        if let Some(v) = parse_u16("c_min") {
            criteria.c_min = v;
        }
        if let Some(v) = parse_u16("c_max") {
            criteria.c_max = v;
        }
        if let Some(v) = parse_u16("h_min") {
            criteria.h_min = v;
        }
        if let Some(v) = parse_u16("h_max") {
            criteria.h_max = v;
        }
        if let Some(v) = parse_u16("n_min") {
            criteria.n_min = v;
        }
        if let Some(v) = parse_u16("n_max") {
            criteria.n_max = v;
        }
        if let Some(v) = parse_u16("o_min") {
            criteria.o_min = v;
        }
        if let Some(v) = parse_u16("o_max") {
            criteria.o_max = v;
        }
        if let Some(v) = parse_u16("p_min") {
            criteria.p_min = v;
        }
        if let Some(v) = parse_u16("p_max") {
            criteria.p_max = v;
        }
        if let Some(v) = parse_u16("s_min") {
            criteria.s_min = v;
        }
        if let Some(v) = parse_u16("s_max") {
            criteria.s_max = v;
        }
        if let Some(v) = params.get("f_state") {
            criteria.f_state = ElementState::from_str(v);
        }
        if let Some(v) = params.get("cl_state") {
            criteria.cl_state = ElementState::from_str(v);
        }
        if let Some(v) = params.get("br_state") {
            criteria.br_state = ElementState::from_str(v);
        }
        if let Some(v) = params.get("i_state") {
            criteria.i_state = ElementState::from_str(v);
        }
    }

    // Share links with only structure should not inherit default taxon.
    if has_structure && !has_explicit_taxon {
        criteria.taxon.clear();
    }

    criteria
}

pub fn build_shareable_url(criteria: &SearchCriteria) -> Option<String> {
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

pub fn absolute_share_url(share: &str) -> String {
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

pub fn absolute_current_url_with_query(query: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            let loc = win.location();
            if let (Ok(origin), Ok(pathname)) = (loc.origin(), loc.pathname()) {
                return format!("{origin}{pathname}?{query}");
            }
        }
    }
    format!("?{query}")
}

pub fn initial_view_from_url() -> AppView {
    let params = read_url_query_params();
    AppView::from_query_value(params.get("view").map(String::as_str))
}

pub fn initial_locale_from_url() -> Locale {
    let params = read_url_query_params();
    let lang = params.get("lang").map(|v| v.as_str()).unwrap_or("");
    Locale::detect(lang)
}

pub fn initial_download_format_from_url() -> Option<String> {
    let params = read_url_query_params();
    let (download, _execute) = parse_startup_action_from_params(&params);
    download
}

pub fn initial_execute_from_url() -> bool {
    let params = read_url_query_params();
    let (_download, execute) = parse_startup_action_from_params(&params);
    execute
}

pub fn persist_locale_query_param(locale: Locale) {
    #[cfg(target_arch = "wasm32")]
    {
        let mut params = read_url_query_params();
        params.insert(
            "lang".to_string(),
            match locale {
                Locale::En => "en",
                Locale::Fr => "fr",
                Locale::De => "de",
                Locale::It => "it",
            }
            .to_string(),
        );
        let query = build_query_string(&params);
        let url = absolute_current_url_with_query(&query);
        if let Some(win) = web_sys::window()
            && let Ok(history) = win.history()
        {
            let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = locale;
    }
}

pub fn persist_view_query_param(view: AppView) {
    #[cfg(target_arch = "wasm32")]
    {
        let mut params = read_url_query_params();
        if let Some(view_param) = view.query_value() {
            params.insert("view".to_string(), view_param.to_string());
        } else {
            params.remove("view");
        }
        let query = build_query_string(&params);
        let url = if query.is_empty() {
            absolute_current_url_with_query("")
                .trim_end_matches('?')
                .to_string()
        } else {
            absolute_current_url_with_query(&query)
        };
        if let Some(win) = web_sys::window()
            && let Ok(history) = win.history()
        {
            let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = view;
    }
}

pub fn parse_startup_action_from_params(
    params: &BTreeMap<String, String>,
) -> (Option<String>, bool) {
    let wants_download = params
        .get("download")
        .map(|v| is_true_flag(v))
        .unwrap_or(false);
    if !wants_download {
        let wants_execute = params
            .get("execute")
            .map(|v| is_true_flag(v))
            .unwrap_or(false);
        return (None, wants_execute);
    }
    (
        Some(
            params
                .get("format")
                .map(|v| v.to_ascii_lowercase())
                .unwrap_or_else(|| "csv".to_string()),
        ),
        false,
    )
}

pub fn is_true_flag(v: &str) -> bool {
    matches!(
        v.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

pub fn read_url_query_params() -> BTreeMap<String, String> {
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

#[cfg(target_arch = "wasm32")]
fn build_query_string(params: &BTreeMap<String, String>) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
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
        let (download, execute) = parse_startup_action_from_params(&params);
        assert!(download.is_none());
        assert!(execute);
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
    fn startup_action_download_has_priority_over_execute() {
        let mut params = BTreeMap::new();
        params.insert("download".into(), "yes".into());
        params.insert("execute".into(), "true".into());
        params.insert("format".into(), "rdf".into());
        let (download, execute) = parse_startup_action_from_params(&params);
        assert_eq!(download.as_deref(), Some("rdf"));
        assert!(!execute);
    }
}
