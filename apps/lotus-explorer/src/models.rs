//! Domain models for the LOTUS explorer.

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum rows returned by a single SPARQL query.
///
/// Mirrors the Python `CONFIG["table_row_limit"]` which caps at 100 in
/// Pyodide/WASM (to preserve memory / keep the UI snappy) and 1000 otherwise.
#[cfg(target_arch = "wasm32")]
pub const TABLE_ROW_LIMIT: usize = 1_000_000;
#[cfg(not(target_arch = "wasm32"))]
pub const TABLE_ROW_LIMIT: usize = 1_000_000;

// Default element-range ceilings. Values at or within
// `[0, DEFAULT_*_MAX]` are considered "inactive" (Python: `min_val == 0 &&
// max_val is None`).
pub const DEFAULT_C_MAX: i32 = 100;
pub const DEFAULT_H_MAX: i32 = 200;
pub const DEFAULT_N_MAX: i32 = 50;
pub const DEFAULT_O_MAX: i32 = 50;
pub const DEFAULT_P_MAX: i32 = 20;
pub const DEFAULT_S_MAX: i32 = 20;

/// Earliest supported publication year (matches Python).
pub const DEFAULT_YEAR_MIN: i32 = 1900;

/// Current calendar year, computed at runtime. Matches Python's
/// `datetime.now().year`. On WASM we ask the browser; on native we derive
/// from the system clock.
pub fn current_year() -> i32 {
    #[cfg(target_arch = "wasm32")]
    {
        // `js_sys::Date` is re-exported by `web-sys`'s `js-sys` dependency.
        js_sys::Date::new_0().get_full_year() as i32
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        // 365.2425 days/year ≈ 31_556_952 s/year.
        (1970 + secs / 31_556_952) as i32
    }
}

// ── Compound entry ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompoundEntry {
    pub compound_qid: String,
    pub name: String,
    pub inchikey: Option<String>,
    pub smiles: Option<String>,
    pub mass: Option<f64>,
    pub formula: Option<String>,
    pub taxon_qid: String,
    pub taxon_name: String,
    pub reference_qid: String,
    pub ref_title: Option<String>,
    pub ref_doi: Option<String>,
    pub pub_year: Option<i32>,
    pub statement: Option<String>,
}

impl CompoundEntry {
    pub fn compound_url(&self) -> String {
        format!("https://www.wikidata.org/entity/{}", self.compound_qid)
    }
    pub fn taxon_url(&self) -> String {
        format!("https://www.wikidata.org/entity/{}", self.taxon_qid)
    }
    pub fn reference_url(&self) -> String {
        format!("https://www.wikidata.org/entity/{}", self.reference_qid)
    }
    pub fn scholia_url(&self) -> String {
        format!(
            "https://scholia.toolforge.org/chemical/{}",
            self.compound_qid
        )
    }
    pub fn doi_url(&self) -> Option<String> {
        self.ref_doi
            .as_ref()
            .map(|d| format!("https://doi.org/{d}"))
    }

    /// CDK Depict SVG URL for the compound's SMILES. Matches Python
    /// `svg_from_smiles` (layout = `cow`, format = `svg`, annotate = `cip`).
    pub fn depict_url(&self) -> Option<String> {
        let smiles = self.smiles.as_ref()?.trim();
        if smiles.is_empty() || smiles.contains('\n') {
            return None;
        }
        Some(format!(
            "https://www.simolecule.com/cdkdepict/depict/cow/svg?smi={}&annotate=cip",
            urlencoding::encode(smiles)
        ))
    }

    pub fn statement_id(&self) -> Option<String> {
        self.statement
            .as_ref()
            .map(|s| s.replace("http://www.wikidata.org/entity/statement/", ""))
            .filter(|s| !s.trim().is_empty())
    }

    pub fn statement_url(&self) -> Option<String> {
        self.statement_id()
            .map(|id| format!("https://www.wikidata.org/entity/statement/{id}"))
    }
}

// ── Search criteria ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct SearchCriteria {
    pub taxon: String,
    pub smiles: String,
    pub smiles_search_type: SmilesSearchType,
    pub smiles_threshold: f64,
    pub mass_min: f64,
    pub mass_max: f64,
    pub year_min: i32,
    pub year_max: i32,
    pub formula_enabled: bool,
    pub formula_exact: String,
    pub c_min: i32,
    pub c_max: i32,
    pub h_min: i32,
    pub h_max: i32,
    pub n_min: i32,
    pub n_max: i32,
    pub o_min: i32,
    pub o_max: i32,
    pub p_min: i32,
    pub p_max: i32,
    pub s_min: i32,
    pub s_max: i32,
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
            mass_max: 2000.0,
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
        self.mass_min > 0.0 || self.mass_max < 2000.0
    }
    pub fn has_year_filter(&self) -> bool {
        self.year_min > DEFAULT_YEAR_MIN || self.year_max < current_year()
    }

    /// A per-element range is "active" only when the user has moved it off
    /// its default span (Python: `ElementRange.is_active`).
    pub fn element_ranges(&self) -> [(&'static str, i32, i32, i32); 6] {
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
            if !self.smiles.contains('\n') && !self.smiles.contains('\r') {
                params.push(("smiles".to_string(), self.smiles.clone()));
            }
            params.push((
                "smiles_search_type".to_string(),
                self.smiles_search_type.as_str().to_string(),
            ));
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
        params
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SmilesSearchType {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ElementState {
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

    pub fn from_str(value: &str) -> Self {
        match value {
            "required" => Self::Required,
            "excluded" => Self::Excluded,
            _ => Self::Allowed,
        }
    }
}

// ── Dataset statistics ────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub struct DatasetStats {
    pub n_compounds: usize,
    pub n_taxa: usize,
    pub n_references: usize,
    pub n_entries: usize,
}

impl DatasetStats {
    pub fn from_entries(entries: &[CompoundEntry]) -> Self {
        use std::collections::HashSet;
        Self {
            n_compounds: entries
                .iter()
                .map(|e| e.compound_qid.as_str())
                .collect::<HashSet<_>>()
                .len(),
            n_taxa: entries
                .iter()
                .map(|e| e.taxon_qid.as_str())
                .collect::<HashSet<_>>()
                .len(),
            n_references: entries
                .iter()
                .map(|e| e.reference_qid.as_str())
                .collect::<HashSet<_>>()
                .len(),
            n_entries: entries.len(),
        }
    }
}

// ── Taxon resolution ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TaxonMatch {
    pub qid: String,
    pub name: String,
}

// ── Sort state ────────────────────────────────────────────────────────────────

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

#[derive(Debug, Clone)]
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
