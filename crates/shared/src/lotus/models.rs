// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Domain models for the LOTUS explorer/API shared core.
//!
//! ## Linked Open Data / Wikidata
//!
//! All entity identifiers in the LOTUS dataset follow the Wikidata entity URI
//! scheme.  The canonical prefix is [`WIKIDATA_ENTITY_BASE`].  Statement
//! identifiers use [`WIKIDATA_STATEMENT_BASE`].  These constants are
//! re-exported here so every layer (DTO deserialization, SPARQL parsing, UI
//! display) uses a single authoritative value.

use std::sync::Arc;

/// Base URI for Wikidata entities (e.g. `Q12345` → `<BASE>Q12345`).
pub const WIKIDATA_ENTITY_BASE: &str = "http://www.wikidata.org/entity/";

/// Base URI for Wikidata reification statements.
pub const WIKIDATA_STATEMENT_BASE: &str = "http://www.wikidata.org/entity/statement/";

#[cfg(target_arch = "wasm32")]
pub const TABLE_ROW_LIMIT: usize = 1_000;
#[cfg(not(target_arch = "wasm32"))]
pub const TABLE_ROW_LIMIT: usize = 2_000_000;

pub const DEFAULT_C_MAX: u16 = 512;
pub const DEFAULT_H_MAX: u16 = 1_024;
pub const DEFAULT_N_MAX: u16 = 256;
pub const DEFAULT_O_MAX: u16 = 256;
pub const DEFAULT_P_MAX: u16 = 128;
pub const DEFAULT_S_MAX: u16 = 64;

pub const DEFAULT_YEAR_MIN: u16 = 1800;

pub type Rows = Arc<[CompoundEntry]>;

pub fn runtime_table_row_limit() -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        // Keep wasm conservative by default while still scaling on capable devices.
        let mut limit = 500usize;
        if let Some(win) = web_sys::window() {
            let win_js = wasm_bindgen::JsValue::from(win);
            if let Ok(nav) =
                js_sys::Reflect::get(&win_js, &wasm_bindgen::JsValue::from_str("navigator"))
            {
                if let Ok(mem) =
                    js_sys::Reflect::get(&nav, &wasm_bindgen::JsValue::from_str("deviceMemory"))
                    && let Some(gb) = mem.as_f64()
                {
                    if gb <= 2.0 {
                        limit = 220;
                    } else if gb <= 4.0 {
                        limit = 360;
                    } else if gb >= 8.0 {
                        limit = 800;
                    }
                }

                if let Ok(ua) =
                    js_sys::Reflect::get(&nav, &wasm_bindgen::JsValue::from_str("userAgent"))
                    && let Some(ua) = ua.as_string()
                {
                    let ua = ua.to_ascii_lowercase();
                    let mobile = ua.contains("iphone")
                        || ua.contains("ipad")
                        || ua.contains("android")
                        || ua.contains("mobile");
                    if mobile {
                        limit = limit.min(280);
                    }
                }
            }
        }
        limit.clamp(180, TABLE_ROW_LIMIT)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        TABLE_ROW_LIMIT
    }
}

pub fn current_year() -> u16 {
    use std::sync::OnceLock;
    static CACHE: OnceLock<u16> = OnceLock::new();
    *CACHE.get_or_init(|| {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::Date::new_0().get_full_year().min(u16::MAX as u32) as u16
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            (1970 + secs / 31_556_952).clamp(0, u16::MAX as i64) as u16
        }
    })
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompoundEntry {
    pub compound_qid: Arc<str>,
    pub name: Arc<str>,
    pub inchikey: Option<Arc<str>>,
    pub smiles: Option<Arc<str>>,
    pub mass: Option<f64>,
    pub formula: Option<Arc<str>>,
    pub taxon_qid: Arc<str>,
    pub taxon_name: Arc<str>,
    pub reference_qid: Arc<str>,
    pub ref_title: Option<Arc<str>>,
    pub ref_doi: Option<Arc<str>>,
    pub pub_year: Option<i16>,
    pub statement: Option<Arc<str>>,
}

impl CompoundEntry {
    pub fn doi(&self) -> Option<&str> {
        self.ref_doi
            .as_deref()
            .map(str::trim)
            .filter(|d| !d.is_empty())
    }

    pub fn doi_url(&self) -> Option<String> {
        self.doi().map(|d| format!("https://doi.org/{d}"))
    }

    pub fn depict_url(&self) -> Option<String> {
        let smiles = self.smiles.as_deref()?.trim();
        if smiles.is_empty() || smiles.contains('\n') {
            return None;
        }
        Some(format!(
            "https://www.simolecule.com/cdkdepict/depict/cow/svg?smi={}&annotate=cip",
            urlencoding::encode(smiles)
        ))
    }

    pub fn statement_id_str(&self) -> Option<&str> {
        let raw = self.statement.as_deref().map(str::trim)?;
        if raw.is_empty() {
            return None;
        }
        Some(raw.strip_prefix(WIKIDATA_STATEMENT_BASE).unwrap_or(raw))
    }

    pub fn statement_id(&self) -> Option<String> {
        self.statement_id_str().map(str::to_owned)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchCriteria {
    pub taxon: String,
    pub smiles: String,
    pub smiles_search_type: SmilesSearchType,
    pub smiles_threshold: f64,
    pub mass_min: f64,
    pub mass_max: f64,
    pub year_min: u16,
    pub year_max: u16,
    pub formula_enabled: bool,
    pub formula_exact: String,
    pub c_min: u16,
    pub c_max: u16,
    pub h_min: u16,
    pub h_max: u16,
    pub n_min: u16,
    pub n_max: u16,
    pub o_min: u16,
    pub o_max: u16,
    pub p_min: u16,
    pub p_max: u16,
    pub s_min: u16,
    pub s_max: u16,
    pub f_state: ElementState,
    pub cl_state: ElementState,
    pub br_state: ElementState,
    pub i_state: ElementState,
}

impl Default for SearchCriteria {
    fn default() -> Self {
        Self {
            taxon: "Gentiana lutea".into(),
            smiles: String::new(),
            smiles_search_type: SmilesSearchType::Substructure,
            smiles_threshold: 0.8,
            mass_min: 0.0,
            mass_max: 10000.0,
            year_min: DEFAULT_YEAR_MIN,
            year_max: current_year(),
            formula_enabled: false,
            formula_exact: String::new(),
            c_min: 0,
            c_max: DEFAULT_C_MAX,
            h_min: 0,
            h_max: DEFAULT_H_MAX,
            n_min: 0,
            n_max: DEFAULT_N_MAX,
            o_min: 0,
            o_max: DEFAULT_O_MAX,
            p_min: 0,
            p_max: DEFAULT_P_MAX,
            s_min: 0,
            s_max: DEFAULT_S_MAX,
            f_state: ElementState::Allowed,
            cl_state: ElementState::Allowed,
            br_state: ElementState::Allowed,
            i_state: ElementState::Allowed,
        }
    }
}

impl SearchCriteria {
    pub fn has_mass_filter(&self) -> bool {
        self.mass_min > 0.0 || self.mass_max < 10000.0
    }

    pub fn has_year_filter(&self) -> bool {
        self.year_min > DEFAULT_YEAR_MIN || self.year_max < current_year()
    }

    pub fn element_ranges(&self) -> [(&'static str, u16, u16, u16); 6] {
        [
            ("C", self.c_min, self.c_max, DEFAULT_C_MAX),
            ("H", self.h_min, self.h_max, DEFAULT_H_MAX),
            ("N", self.n_min, self.n_max, DEFAULT_N_MAX),
            ("O", self.o_min, self.o_max, DEFAULT_O_MAX),
            ("P", self.p_min, self.p_max, DEFAULT_P_MAX),
            ("S", self.s_min, self.s_max, DEFAULT_S_MAX),
        ]
    }

    pub fn has_formula_filter(&self) -> bool {
        self.formula_enabled
            && (!self.formula_exact.trim().is_empty()
                || self
                    .element_ranges()
                    .iter()
                    .any(|(_, min, max, default_max)| *min > 0 || *max < *default_max)
                || self.f_state != ElementState::Allowed
                || self.cl_state != ElementState::Allowed
                || self.br_state != ElementState::Allowed
                || self.i_state != ElementState::Allowed)
    }

    pub fn has_effective_filters(&self) -> bool {
        !self.smiles.trim().is_empty()
            || self.has_mass_filter()
            || self.has_year_filter()
            || self.has_formula_filter()
    }

    pub fn is_valid(&self) -> bool {
        !self.taxon.trim().is_empty() || !self.smiles.trim().is_empty()
    }

    pub fn shareable_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if !self.taxon.trim().is_empty() {
            params.push(("taxon".to_string(), self.taxon.clone()));
        }
        if !self.smiles.trim().is_empty() {
            params.push(("structure".to_string(), self.smiles.clone()));
            params.push((
                "structure_search_type".to_string(),
                self.smiles_search_type.as_str().to_string(),
            ));
            if self.smiles_search_type == SmilesSearchType::Similarity {
                params.push((
                    "smiles_threshold".to_string(),
                    format!("{:.2}", self.smiles_threshold),
                ));
            }
        }
        if self.has_mass_filter() {
            params.push(("mass_filter".to_string(), "true".to_string()));
            params.push(("mass_min".to_string(), format!("{}", self.mass_min)));
            params.push(("mass_max".to_string(), format!("{}", self.mass_max)));
        }
        if self.has_year_filter() {
            params.push(("year_filter".to_string(), "true".to_string()));
            params.push(("year_start".to_string(), format!("{}", self.year_min)));
            params.push(("year_end".to_string(), format!("{}", self.year_max)));
        }
        if self.formula_enabled {
            params.push(("formula_filter".to_string(), "true".to_string()));
            if !self.formula_exact.trim().is_empty() {
                params.push(("formula_exact".to_string(), self.formula_exact.clone()));
            }
            for (label, min, max, default_max) in self.element_ranges() {
                let key = label.to_ascii_lowercase();
                if min > 0 {
                    params.push((format!("{key}_min"), min.to_string()));
                }
                if max < default_max {
                    params.push((format!("{key}_max"), max.to_string()));
                }
            }
            for (label, state) in [
                ("f", self.f_state),
                ("cl", self.cl_state),
                ("br", self.br_state),
                ("i", self.i_state),
            ] {
                if state != ElementState::Allowed {
                    params.push((format!("{label}_state"), state.as_str().to_string()));
                }
            }
        }
        params
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SmilesSearchType {
    #[default]
    Substructure,
    Similarity,
}

impl SmilesSearchType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Substructure => "substructure",
            Self::Similarity => "similarity",
        }
    }
}

impl std::fmt::Display for SmilesSearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ElementState {
    #[default]
    Allowed,
    Required,
    Excluded,
}

impl ElementState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Required => "required",
            Self::Excluded => "excluded",
        }
    }
}

impl std::fmt::Display for ElementState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ElementState {
    type Err = std::convert::Infallible;

    /// Parse a case-sensitive element-state string.
    ///
    /// Accepts `"required"` and `"excluded"`; all other values (including
    /// `"allowed"` and unrecognised strings) map to [`ElementState::Allowed`].
    /// This is intentionally infallible so URL parameters can always be decoded
    /// without propagating parse errors through the call stack.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "required" => Self::Required,
            "excluded" => Self::Excluded,
            _ => Self::Allowed,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DatasetStats {
    pub n_compounds: usize,
    pub n_taxa: usize,
    pub n_references: usize,
    pub n_entries: usize,
    pub n_entries_unique: usize,
}

impl DatasetStats {
    /// Compute dataset statistics from a slice of result entries.
    ///
    /// Uses `&str` slices (borrowed from `Arc<str>` fields) so no strings are
    /// copied or re-allocated.  A single pass over the entries populates all
    /// deduplicated ID sets simultaneously, including unique
    /// compound-taxon-reference triples (matching the `COUNT(DISTINCT …)`
    /// computed by `query_counts_from_base`).
    pub fn from_entries(entries: &[CompoundEntry]) -> Self {
        use std::collections::HashSet;
        let mut c: HashSet<&str> = HashSet::with_capacity(entries.len());
        let mut t: HashSet<&str> = HashSet::with_capacity(entries.len());
        let mut r: HashSet<&str> = HashSet::with_capacity(entries.len());
        let mut unique_triples: HashSet<(&str, &str, &str)> = HashSet::with_capacity(entries.len());
        for e in entries {
            c.insert(e.compound_qid.as_ref());
            if !e.taxon_qid.is_empty() {
                t.insert(e.taxon_qid.as_ref());
            }
            if !e.reference_qid.is_empty() {
                r.insert(e.reference_qid.as_ref());
            }
            unique_triples.insert((
                e.compound_qid.as_ref(),
                e.taxon_qid.as_ref(),
                e.reference_qid.as_ref(),
            ));
        }
        Self {
            n_compounds: c.len(),
            n_taxa: t.len(),
            n_references: r.len(),
            n_entries: entries.len(),
            n_entries_unique: unique_triples.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaxonMatch {
    pub qid: String,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortColumn {
    Name,
    Mass,
    Formula,
    TaxonName,
    PubYear,
    RefTitle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDir {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SortState {
    pub col: SortColumn,
    pub dir: SortDir,
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            col: SortColumn::Name,
            dir: SortDir::Asc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CompoundEntry, DatasetStats, ElementState, SearchCriteria, SmilesSearchType};
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn make_entry(compound: &str, taxon: &str, reference: &str) -> CompoundEntry {
        CompoundEntry {
            compound_qid: Arc::from(compound),
            name: Arc::from(""),
            taxon_qid: Arc::from(taxon),
            taxon_name: Arc::from(""),
            reference_qid: Arc::from(reference),
            ..CompoundEntry::default()
        }
    }

    #[test]
    fn dataset_stats_from_entries_counts_unique_triples() {
        let entries = vec![
            make_entry("Q1", "Q10", "Q100"),
            make_entry("Q1", "Q10", "Q100"), // duplicate triple
            make_entry("Q1", "Q11", "Q101"), // same compound, different taxon+ref
            make_entry("Q2", "Q10", "Q100"),
        ];
        let stats = DatasetStats::from_entries(&entries);
        assert_eq!(stats.n_entries, 4);
        assert_eq!(stats.n_entries_unique, 3); // 3 distinct (compound, taxon, ref) triples
        assert_eq!(stats.n_compounds, 2);
        assert_eq!(stats.n_taxa, 2);
        assert_eq!(stats.n_references, 2);
    }

    #[test]
    fn dataset_stats_from_entries_empty_slice() {
        let stats = DatasetStats::from_entries(&[]);
        assert_eq!(stats.n_entries, 0);
        assert_eq!(stats.n_entries_unique, 0);
        assert_eq!(stats.n_compounds, 0);
    }

    #[test]
    fn dataset_stats_from_entries_all_identical() {
        let entries = vec![make_entry("Q1", "Q1", "Q1"); 5];
        let stats = DatasetStats::from_entries(&entries);
        assert_eq!(stats.n_entries, 5);
        assert_eq!(stats.n_entries_unique, 1);
    }

    #[test]
    fn shareable_query_params_omit_default_formula_values_when_only_toggle_is_enabled() {
        let criteria = SearchCriteria {
            taxon: "Fungi".into(),
            formula_enabled: true,
            ..SearchCriteria::default()
        };

        let params: BTreeMap<String, String> =
            criteria.shareable_query_params().into_iter().collect();

        assert_eq!(params.get("taxon").map(String::as_str), Some("Fungi"));
        assert_eq!(
            params.get("formula_filter").map(String::as_str),
            Some("true")
        );
        for key in [
            "formula_exact",
            "c_min",
            "c_max",
            "h_min",
            "h_max",
            "n_min",
            "n_max",
            "o_min",
            "o_max",
            "p_min",
            "p_max",
            "s_min",
            "s_max",
            "f_state",
            "cl_state",
            "br_state",
            "i_state",
        ] {
            assert!(
                !params.contains_key(key),
                "unexpected default formula param: {key}"
            );
        }
    }

    #[test]
    fn shareable_query_params_keep_only_non_default_formula_overrides() {
        let criteria = SearchCriteria {
            taxon: "Fungi".into(),
            formula_enabled: true,
            c_min: 1,
            c_max: 10,
            o_max: 32,
            cl_state: ElementState::Required,
            br_state: ElementState::Excluded,
            ..SearchCriteria::default()
        };

        let params: BTreeMap<String, String> =
            criteria.shareable_query_params().into_iter().collect();

        assert_eq!(
            params.get("formula_filter").map(String::as_str),
            Some("true")
        );
        assert_eq!(params.get("c_min").map(String::as_str), Some("1"));
        assert_eq!(params.get("c_max").map(String::as_str), Some("10"));
        assert_eq!(params.get("o_max").map(String::as_str), Some("32"));
        assert_eq!(params.get("cl_state").map(String::as_str), Some("required"));
        assert_eq!(params.get("br_state").map(String::as_str), Some("excluded"));
        assert!(!params.contains_key("o_min"));
        assert!(!params.contains_key("f_state"));
        assert!(!params.contains_key("i_state"));
    }

    #[test]
    fn shareable_query_params_use_single_structure_param_namespace() {
        let criteria = SearchCriteria {
            taxon: "Gentiana lutea".into(),
            smiles: "CCCC".into(),
            smiles_search_type: SmilesSearchType::Substructure,
            ..SearchCriteria::default()
        };

        let params: BTreeMap<String, String> =
            criteria.shareable_query_params().into_iter().collect();

        assert_eq!(params.get("structure").map(String::as_str), Some("CCCC"));
        assert_eq!(
            params.get("structure_search_type").map(String::as_str),
            Some("substructure")
        );
        assert!(!params.contains_key("smiles"));
        assert!(!params.contains_key("smiles_search_type"));
    }
}
