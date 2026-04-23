//! SPARQL query builders mirroring the Python `modules/knowledge/wikidata/sparql/` module.
//!
//! Key design choices (from the Python source):
//!
//! * All queries use `STRAFTER(STR(?entity), "http://www.wikidata.org/entity/")` to
//!   return bare QIDs (e.g. `Q12345`) rather than full URIs.
//! * Compound–taxon links use the **statement graph pattern**:
//!   `?compound p:P703 ?stmt . ?stmt ps:P703 ?taxon`
//! * References are reached via `prov:wasDerivedFrom / pr:P248`.
//! * QLever does **not** support `SERVICE wikibase:label`, so labels are fetched
//!   with an explicit `rdfs:label` triple filtered to `LANG = "en"`.
//! * Taxon hierarchy is traversed with `wdt:P171*` (parent taxon, transitive).
//! * **No `LIMIT` clause** — Python does not limit the query either. QLever streams
//!   CSV extremely fast; the display cap (see `models::TABLE_ROW_LIMIT`) is applied
//!   client-side after parsing so users can export the full result set.

use crate::models::SmilesSearchType;

// ── Common SPARQL prefixes ────────────────────────────────────────────────────

const PREFIXES: &str = r#"
PREFIX wd:     <http://www.wikidata.org/entity/>
PREFIX wdt:    <http://www.wikidata.org/prop/direct/>
PREFIX p:      <http://www.wikidata.org/prop/>
PREFIX ps:     <http://www.wikidata.org/prop/statement/>
PREFIX pq:     <http://www.wikidata.org/prop/qualifier/>
PREFIX pr:     <http://www.wikidata.org/prop/reference/>
PREFIX prov:   <http://www.w3.org/ns/prov#>
PREFIX rdfs:   <http://www.w3.org/2000/01/rdf-schema#>
PREFIX wikibase: <http://wikiba.se/ontology#>
PREFIX xsd:    <http://www.w3.org/2001/XMLSchema#>
PREFIX schema: <http://schema.org/>
"#;

// ── SELECT columns shared by all compound queries ─────────────────────────────

const COMPOUND_SELECT: &str = r#"
SELECT
  (xsd:integer(STRAFTER(STR(?c), "Q")) AS ?compound)
  ?compoundLabel
  ?compound_inchikey
  ?compound_smiles_conn
  ?compound_smiles_iso
  ?compound_mass
  ?compound_formula
  (xsd:integer(STRAFTER(STR(?t), "Q")) AS ?taxon)
  ?taxon_name
  (xsd:integer(STRAFTER(STR(?r), "Q")) AS ?ref_qid)
  ?ref
  ?ref_title
  ?ref_doi
  ?ref_date
  ?statement
"#;

// ── Query fragments aligned with Python patterns_compound.py ─────────────────

const COMPOUND_IDENTIFIERS: &str = r#"
  ?c wdt:P235 ?compound_inchikey;
     wdt:P233 ?compound_smiles_conn.
"#;

const TAXON_REFERENCE_ASSOCIATION: &str = r#"
  ?c p:P703 ?statement.
  ?statement ps:P703 ?t;
             prov:wasDerivedFrom ?ref.
  ?ref pr:P248 ?r.
  ?t wdt:P225 ?taxon_name.
"#;

const REFERENCE_METADATA_OPTIONAL: &str = r#"
  OPTIONAL { ?r wdt:P1476 ?ref_title. }
  OPTIONAL { ?r wdt:P356 ?ref_doi. }
  OPTIONAL { ?r wdt:P577 ?ref_date. }
"#;

const PROPERTIES_OPTIONAL: &str = r#"
  OPTIONAL { ?c wdt:P2017 ?compound_smiles_iso. }
  OPTIONAL { ?c wdt:P2067 ?compound_mass. }
  OPTIONAL { ?c wdt:P274 ?compound_formula. }
  OPTIONAL {
    ?c rdfs:label ?compoundLabelMul.
    FILTER(LANG(?compoundLabelMul) = "mul")
  }
  OPTIONAL {
    ?c rdfs:label ?compoundLabelEn.
    FILTER(LANG(?compoundLabelEn) = "en")
  }
  BIND(COALESCE(?compoundLabelMul, ?compoundLabelEn) AS ?compoundLabel)
"#;

// ── query_taxon_search ────────────────────────────────────────────────────────

/// Search for taxa by name label (case-insensitive, English).
/// Returns columns: `taxon` (full URI), `taxon_name`.
///
/// Mirrors Python `query_taxon_search`.
pub fn query_taxon_search(name: &str) -> String {
    let e = name.replace('\\', r"\\").replace('"', r#"\""#);

    format!(
        r#"
PREFIX wdt: <http://www.wikidata.org/prop/direct/>
SELECT
?taxon
?taxon_name WHERE {{
    VALUES ?taxon_name {{ "{e}" }}
    ?taxon wdt:P225 ?taxon_name .
}}
"#
    )
}

// ── query_compounds_by_taxon ──────────────────────────────────────────────────

/// Fetch all compound–taxon–reference triples for a given taxon QID,
/// including all descendant taxa via `wdt:P171*`.
///
/// Mirrors Python `query_compounds_by_taxon` **byte-for-byte on the join
/// structure**. Do not rearrange: on QLever, placing the full compound /
/// statement / reference join inside the inner subquery and filtering by
/// the taxon hierarchy in the outer block is *dramatically* faster
/// (seconds vs. tens of seconds) than the reverse — the planner picks an
/// index-driven compound-first plan and streams the result.
pub fn query_compounds_by_taxon(taxon_qid: &str) -> String {
    format!(
        r#"{PREFIXES}
{COMPOUND_SELECT}
WHERE {{
  {{
    SELECT
      ?c
      ?compound_inchikey
      ?compound_smiles_conn
      ?t
      ?taxon_name
      ?r
      ?ref
      ?statement
    WHERE {{
      {COMPOUND_IDENTIFIERS}
      {TAXON_REFERENCE_ASSOCIATION}
    }}
  }}
  ?t (wdt:P171*) wd:{taxon_qid}.
  {REFERENCE_METADATA_OPTIONAL}
  {PROPERTIES_OPTIONAL}
}}"#
    )
}

// ── query_all_compounds ───────────────────────────────────────────────────────

/// Fetch compound–taxon–reference triples for ALL taxa (the full LOTUS dataset).
///
/// Mirrors Python `query_all_compounds`.
pub fn query_all_compounds() -> String {
    format!(
        r#"{PREFIXES}
{COMPOUND_SELECT}
WHERE {{
  {{
    SELECT
      ?c
      ?compound_inchikey
      ?compound_smiles_conn
      ?t
      ?taxon_name
      ?r
      ?ref
      ?statement
    WHERE {{
      {COMPOUND_IDENTIFIERS}
      {TAXON_REFERENCE_ASSOCIATION}
    }}
  }}
  {REFERENCE_METADATA_OPTIONAL}
  {PROPERTIES_OPTIONAL}
}}"#
    )
}

// ── query_sachem ──────────────────────────────────────────────────────────────

/// Fetch compounds matching a SMILES query via the SACHEM/IDSM service,
/// optionally restricted to a given taxon and its descendants.
///
/// Mirrors Python `query_sachem`.
pub fn query_sachem(
    smiles: &str,
    search_type: SmilesSearchType,
    threshold: f64,
    taxon_qid: Option<&str>,
) -> String {
    // Python validate_and_escape: wrap multiline / Molfile blocks (only
    // backslashes need escaping there). Single-line SMILES are wrapped in "…" with
    // backslashes and double-quotes escaped. The emitted string includes the quotes.
    let structure_literal = escape_structure_literal(smiles);
    let is_multiline_literal =
        structure_literal.starts_with("'''") || structure_literal.starts_with(r#"""""#);

    let sachem_clause = match search_type {
        SmilesSearchType::Similarity => format!(
            r#"SERVICE idsm:wikidata {{
    ?c sachem:similarCompoundSearch [
      sachem:query {structure_literal};
      sachem:cutoff "{threshold}"^^xsd:double
    ].
  }}"#
        ),
        SmilesSearchType::Substructure if is_multiline_literal => format!(
            r#"SERVICE idsm:wikidata {{
    [ sachem:compound ?c; sachem:score ?_sachem_score ]
      sachem:scoredSubstructureSearch [
        sachem:query {structure_literal};
        sachem:searchMode sachem:substructureSearch;
        sachem:chargeMode sachem:defaultChargeAsAny;
        sachem:isotopeMode sachem:ignoreIsotopes;
        sachem:aromaticityMode sachem:aromaticityDetectIfMissing;
        sachem:stereoMode sachem:ignoreStereo;
        sachem:tautomerMode sachem:ignoreTautomers;
        sachem:radicalMode sachem:ignoreSpinMultiplicity;
        sachem:topn "-1"^^xsd:integer;
        sachem:internalMatchingLimit "1000000"^^xsd:integer
      ].
  }}"#
        ),
        SmilesSearchType::Substructure => format!(
            r#"SERVICE idsm:wikidata {{
    ?c sachem:substructureSearch [
      sachem:query {structure_literal}
    ].
  }}"#
        ),
    };

    let body = if let Some(qid) = taxon_qid {
        format!(
            r#"
  {sachem_clause}
  {COMPOUND_IDENTIFIERS}

  ?c p:P703 ?statement .
  ?statement ps:P703 ?t ;
             prov:wasDerivedFrom ?ref .
  ?ref pr:P248 ?r .
  ?t wdt:P225 ?taxon_name .
  ?t (wdt:P171*) wd:{qid} .

  {REFERENCE_METADATA_OPTIONAL}
  {PROPERTIES_OPTIONAL}
"#
        )
    } else {
        format!(
            r#"
  {sachem_clause}
  {COMPOUND_IDENTIFIERS}

  OPTIONAL {{
    ?c p:P703 ?statement .
    ?statement ps:P703 ?t ;
               prov:wasDerivedFrom ?ref .
    ?ref pr:P248 ?r .
    ?t wdt:P225 ?taxon_name .
    {REFERENCE_METADATA_OPTIONAL}
  }}

  {PROPERTIES_OPTIONAL}
"#
        )
    };

    format!(
        r#"{PREFIXES}
PREFIX sachem: <http://bioinfo.uochb.cas.cz/rdf/v1.0/sachem#>
PREFIX idsm:   <https://idsm.elixir-czech.cz/sparql/endpoint/>
{COMPOUND_SELECT}
WHERE {{
{body}
}}"#
    )
}

// ── Structure-literal escaping (mirrors Python validate_and_escape) ──────────

/// Detected input format for the structure textarea.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureKind {
    /// Empty / whitespace only.
    Empty,
    /// Single-line SMILES.
    Smiles,
    /// Molfile V2000 block (detected by `M  END` + `V2000`).
    MolfileV2000,
    /// Molfile V3000 block (detected by `M  END` + `V3000` / `BEGIN CTAB`).
    MolfileV3000,
}

impl StructureKind {
    /// Short human label used in the UI hint.
    pub fn label(self) -> &'static str {
        match self {
            Self::Empty => "—",
            Self::Smiles => "SMILES",
            Self::MolfileV2000 => "Molfile V2000",
            Self::MolfileV3000 => "Molfile V3000",
        }
    }
}

/// Classify the raw contents of the structure input. Mirrors the Python
/// `_looks_like_molfile` heuristic (matches both V2000 and V3000 CTAB
/// blocks) but additionally distinguishes the two versions so the UI can
/// show which one the user just pasted.
pub fn classify_structure(text: &str) -> StructureKind {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return StructureKind::Empty;
    }
    let upper = text.to_ascii_uppercase();
    let has_end = upper.contains("M  END");
    if has_end && (upper.contains("V3000") || upper.contains("BEGIN CTAB")) {
        return StructureKind::MolfileV3000;
    }
    if has_end && upper.contains("V2000") {
        return StructureKind::MolfileV2000;
    }
    StructureKind::Smiles
}

/// Back-compat helper — returns `true` for either Molfile version.
fn looks_like_molfile(text: &str) -> bool {
    matches!(
        classify_structure(text),
        StructureKind::MolfileV2000 | StructureKind::MolfileV3000
    )
}

/// Build a SPARQL string literal (including surrounding quotes) for a SMILES
/// or Molfile block. Multiline inputs and Molfile blocks are wrapped in
/// triple single-quotes so embedded newlines and double-quotes
/// are preserved as-is. Single-line SMILES use double-quote form with
/// backslash/quote escaping. Matches `validate_and_escape` in Python.
pub fn escape_structure_literal(smiles: &str) -> String {
    let normalized = smiles.replace("\r\n", "\n").replace('\r', "\n");
    let is_molfile = looks_like_molfile(&normalized);
    let candidate = if is_molfile {
        normalized
    } else {
        normalized.trim().to_string()
    };

    let escaped_bs = candidate.replace('\\', r"\\");
    if is_molfile || candidate.contains('\n') {
        format!("'''{escaped_bs}'''")
    } else {
        let escaped = escaped_bs.replace('"', r#"\""#);
        format!("\"{escaped}\"")
    }
}

pub fn query_counts_from_base(base_query: &str) -> String {
    let Some(select_pos) = base_query.find("SELECT") else {
        return base_query.to_string();
    };
    let prefixes = &base_query[..select_pos];
    let inner_select = base_query[select_pos..].trim();

    format!(
        r#"{prefixes}
SELECT
  (COUNT(*) AS ?n_entries)
  (COUNT(DISTINCT ?compound) AS ?n_compounds)
  (COUNT(DISTINCT ?taxon) AS ?n_taxa)
  (COUNT(DISTINCT ?ref_qid) AS ?n_references)
WHERE {{
  {{
    {inner_select}
  }}
}}"#
    )
}

pub fn query_with_limit(base_query: &str, limit: usize) -> String {
    let trimmed = base_query.trim_end();
    format!("{trimmed}\nLIMIT {limit}")
}

pub fn query_with_client_prefilters(
    base_query: &str,
    mass_filter: Option<(f64, f64)>,
    year_filter: Option<(i32, i32)>,
    formula_exact: Option<&str>,
) -> String {
    let mut filters = Vec::new();
    if let Some((min, max)) = mass_filter {
        filters.push(format!(
            "FILTER(BOUND(?compound_mass) && ?compound_mass >= {min:.6} && ?compound_mass <= {max:.6})"
        ));
    }
    if let Some((start, end)) = year_filter {
        filters.push(format!(
            "FILTER(BOUND(?ref_date) && YEAR(?ref_date) >= {start} && YEAR(?ref_date) <= {end})"
        ));
    }
    if let Some(exact) = formula_exact.map(str::trim).filter(|s| !s.is_empty()) {
        let exact_ascii = normalize_formula_digits(exact);
        let exact_escaped = exact_ascii.replace('\\', r"\\").replace('"', r#"\""#);
        let exact_subscript = digits_to_subscripts(&exact_ascii);
        let exact_subscript_escaped = exact_subscript.replace('\\', r"\\").replace('"', r#"\""#);
        if exact_subscript_escaped == exact_escaped {
            filters.push(format!(
                "FILTER(BOUND(?compound_formula) && STR(?compound_formula) = \"{exact_escaped}\")"
            ));
        } else {
            filters.push(format!(
                "FILTER(BOUND(?compound_formula) && (STR(?compound_formula) = \"{exact_escaped}\" || STR(?compound_formula) = \"{exact_subscript_escaped}\"))"
            ));
        }
    }
    if filters.is_empty() {
        return base_query.to_string();
    }

    let trimmed = base_query.trim_end();
    let Some(last_close) = trimmed.rfind('}') else {
        return format!("{trimmed}\n{}", filters.join("\n"));
    };

    let mut out = String::with_capacity(trimmed.len() + filters.len() * 90);
    out.push_str(&trimmed[..last_close]);
    out.push('\n');
    out.push_str(&filters.join("\n"));
    out.push('\n');
    out.push_str(&trimmed[last_close..]);
    out
}

fn normalize_formula_digits(s: &str) -> String {
    s.chars()
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

fn digits_to_subscripts(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '0' => '₀',
            '1' => '₁',
            '2' => '₂',
            '3' => '₃',
            '4' => '₄',
            '5' => '₅',
            '6' => '₆',
            '7' => '₇',
            '8' => '₈',
            '9' => '₉',
            _ => c,
        })
        .collect()
}
