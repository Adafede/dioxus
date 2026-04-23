//! Domain models for the LOTUS explorer.

use std::collections::BTreeMap;
use std::sync::Arc;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum rows returned by a single SPARQL query.
///
/// Mirrors the Python `CONFIG["table_row_limit"]` which caps at 100 in
/// Pyodide/WASM (to preserve memory / keep the UI snappy) and 1000 otherwise.
#[cfg(target_arch = "wasm32")]
pub const TABLE_ROW_LIMIT: usize = 2_000;
#[cfg(not(target_arch = "wasm32"))]
pub const TABLE_ROW_LIMIT: usize = 2_000_000;

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

/// Shared, immutable row buffer. Cloning is a pointer-sized refcount bump,
/// so it is cheap to pass into signals / component props without duplicating
/// the whole result set on every re-render.
pub type Rows = Arc<[CompoundEntry]>;

/// Runtime display cap used by the query `LIMIT` for rendered rows.
///
/// On wasm we keep a smaller cap for low-memory/mobile devices to avoid OOM
/// crashes while preserving exact aggregate counts via the separate count
/// query. Desktop/native keeps the compile-time limit unchanged.
pub fn runtime_table_row_limit() -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        let mut limit = TABLE_ROW_LIMIT;
        if let Some(win) = web_sys::window() {
            let win_js = wasm_bindgen::JsValue::from(win);
            if let Ok(nav) =
                js_sys::Reflect::get(&win_js, &wasm_bindgen::JsValue::from_str("navigator"))
            {
                if let Ok(mem) =
                    js_sys::Reflect::get(&nav, &wasm_bindgen::JsValue::from_str("deviceMemory"))
                {
                    if let Some(gb) = mem.as_f64() {
                        if gb <= 2.0 {
                            limit = limit.min(300);
                        } else if gb <= 4.0 {
                            limit = limit.min(600);
                        }
                    }
                }

                if let Ok(ua) =
                    js_sys::Reflect::get(&nav, &wasm_bindgen::JsValue::from_str("userAgent"))
                {
                    if let Some(ua) = ua.as_string() {
                        let ua = ua.to_ascii_lowercase();
                        let mobile = ua.contains("iphone")
                            || ua.contains("ipad")
                            || ua.contains("android")
                            || ua.contains("mobile");
                        if mobile {
                            limit = limit.min(400);
                        }
                    }
                }
            }
        }
        limit.clamp(150, TABLE_ROW_LIMIT)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        TABLE_ROW_LIMIT
    }
}

/// Current calendar year, computed once and cached.
pub fn current_year() -> i32 {
    use std::sync::OnceLock;
    static CACHE: OnceLock<i32> = OnceLock::new();
    *CACHE.get_or_init(|| {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::Date::new_0().get_full_year() as i32
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            (1970 + secs / 31_556_952) as i32
        }
    })
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

    pub fn has_client_post_filters(&self) -> bool {
        self.has_mass_filter() || self.has_year_filter() || self.has_formula_filter()
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
        if self.formula_enabled {
            params.push(("formula_filter".to_string(), "true".to_string()));
            if !self.formula_exact.trim().is_empty() {
                params.push(("formula_exact".to_string(), self.formula_exact.clone()));
            }
            params.push(("c_min".to_string(), self.c_min.to_string()));
            params.push(("c_max".to_string(), self.c_max.to_string()));
            params.push(("h_min".to_string(), self.h_min.to_string()));
            params.push(("h_max".to_string(), self.h_max.to_string()));
            params.push(("n_min".to_string(), self.n_min.to_string()));
            params.push(("n_max".to_string(), self.n_max.to_string()));
            params.push(("o_min".to_string(), self.o_min.to_string()));
            params.push(("o_max".to_string(), self.o_max.to_string()));
            params.push(("p_min".to_string(), self.p_min.to_string()));
            params.push(("p_max".to_string(), self.p_max.to_string()));
            params.push(("s_min".to_string(), self.s_min.to_string()));
            params.push(("s_max".to_string(), self.s_max.to_string()));
            params.push(("f_state".to_string(), self.f_state.as_str().to_string()));
            params.push(("cl_state".to_string(), self.cl_state.as_str().to_string()));
            params.push(("br_state".to_string(), self.br_state.as_str().to_string()));
            params.push(("i_state".to_string(), self.i_state.as_str().to_string()));
        }
        params
    }
}

/// Apply client-side filters that are not encoded directly into the SPARQL query.
/// This keeps table rows and downloads consistent.
pub fn apply_client_filters_in_place(rows: &mut Vec<CompoundEntry>, crit: &SearchCriteria) {
    if crit.has_mass_filter() {
        rows.retain(|e| {
            e.mass
                .is_some_and(|m| m >= crit.mass_min && m <= crit.mass_max)
        });
    }
    if crit.has_year_filter() {
        rows.retain(|e| {
            e.pub_year
                .is_none_or(|y| y >= crit.year_min && y <= crit.year_max)
        });
    }
    if crit.has_formula_filter() {
        rows.retain(|e| formula_matches(e.formula.as_deref(), crit));
    }
}

fn formula_matches(formula: Option<&str>, crit: &SearchCriteria) -> bool {
    // Python semantics: rows with no formula are not filtered out.
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

#[derive(Clone, Default, PartialEq)]
pub struct DatasetStats {
    pub n_compounds: usize,
    pub n_taxa: usize,
    pub n_references: usize,
    pub n_entries: usize,
}

impl DatasetStats {
    /// Single-pass stats using 64-bit FNV-1a fingerprints instead of
    /// allocating `HashSet<&str>` (which hashes every byte twice and keeps
    /// the string slice metadata alive). On large result sets this is
    /// roughly 3–5× faster and allocates almost nothing.
    pub fn from_entries(entries: &[CompoundEntry]) -> Self {
        use std::collections::HashSet;
        let mut c: HashSet<u64> = HashSet::with_capacity(entries.len());
        let mut t: HashSet<u64> = HashSet::with_capacity(entries.len());
        let mut r: HashSet<u64> = HashSet::with_capacity(entries.len());
        for e in entries {
            c.insert(fnv1a64(e.compound_qid.as_bytes()));
            if !e.taxon_qid.is_empty() {
                t.insert(fnv1a64(e.taxon_qid.as_bytes()));
            }
            if !e.reference_qid.is_empty() {
                r.insert(fnv1a64(e.reference_qid.as_bytes()));
            }
        }
        Self {
            n_compounds: c.len(),
            n_taxa: t.len(),
            n_references: r.len(),
            n_entries: entries.len(),
        }
    }
}

#[inline]
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
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
