// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::models::{
    DEFAULT_C_MAX, DEFAULT_H_MAX, DEFAULT_N_MAX, DEFAULT_O_MAX, DEFAULT_P_MAX, DEFAULT_S_MAX,
    ElementState, SearchCriteria, SmilesSearchType,
};

const SUBSCRIPT_DIGIT_MAPPINGS: [(char, char); 10] = [
    ('₀', '0'),
    ('₁', '1'),
    ('₂', '2'),
    ('₃', '3'),
    ('₄', '4'),
    ('₅', '5'),
    ('₆', '6'),
    ('₇', '7'),
    ('₈', '8'),
    ('₉', '9'),
];

/// Standard Wikidata/QLever SPARQL PREFIX declarations.
/// These prefixes follow W3C ontologies and Wikidata vocabulary standards:
/// - wd: Wikidata entity URIs
/// - wdt: Wikidata direct properties (simplified truthy statements)
/// - p: Wikidata property nodes (full statement structures)
/// - ps: Property statement values
/// - pq: Property qualifiers on statements
/// - pr: Property references on statements
/// - prov: W3C PROV provenance ontology
/// - rdfs: RDF Schema vocabulary
/// - xsd: XML Schema datatypes for typed literals
/// - wikibase: QLever/wikiba.se ontology terms
/// - schema: Schema.org vocabulary
const PREFIXES: &str = r#"PREFIX xsd:    <http://www.w3.org/2001/XMLSchema#>
PREFIX rdfs:   <http://www.w3.org/2000/01/rdf-schema#>
PREFIX prov:   <http://www.w3.org/ns/prov#>
PREFIX wd:     <http://www.wikidata.org/entity/>
PREFIX wdt:    <http://www.wikidata.org/prop/direct/>
PREFIX p:      <http://www.wikidata.org/prop/>
PREFIX ps:     <http://www.wikidata.org/prop/statement/>
PREFIX pq:     <http://www.wikidata.org/prop/qualifier/>
PREFIX pr:     <http://www.wikidata.org/prop/reference/>
PREFIX wikibase: <http://wikiba.se/ontology#>
PREFIX schema: <http://schema.org/>
"#;

/// Extended PREFIXES for structure search queries (Sachem/IDSM service).
/// Includes all standard prefixes plus:
/// - sachem: IDSM Sachem structure search service predicates
/// - idsm: IDSM SPARQL endpoint reference
const PREFIXES_WITH_STRUCTURE: &str = r#"PREFIX xsd:    <http://www.w3.org/2001/XMLSchema#>
PREFIX rdfs:   <http://www.w3.org/2000/01/rdf-schema#>
PREFIX prov:   <http://www.w3.org/ns/prov#>
PREFIX wd:     <http://www.wikidata.org/entity/>
PREFIX wdt:    <http://www.wikidata.org/prop/direct/>
PREFIX p:      <http://www.wikidata.org/prop/>
PREFIX ps:     <http://www.wikidata.org/prop/statement/>
PREFIX pq:     <http://www.wikidata.org/prop/qualifier/>
PREFIX pr:     <http://www.wikidata.org/prop/reference/>
PREFIX wikibase: <http://wikiba.se/ontology#>
PREFIX schema: <http://schema.org/>
PREFIX sachem: <http://bioinfo.uochb.cas.cz/rdf/v1.0/sachem#>
PREFIX idsm:   <https://idsm.elixir-czech.cz/sparql/endpoint/>
"#;

/// Compound identifier retrieval via Wikidata direct properties.
/// P235: InChIKey (canonical chemical fingerprint)
/// P233: SMILES string (canonical SMILES, connection-table form)
const COMPOUND_IDENTIFIERS: &str = r#"
  ?c wdt:P235 ?compound_inchikey;
     wdt:P233 ?compound_smiles_conn.
"#;

/// Taxon-reference association via Wikidata statement structure.
/// P703: Found in taxon (connects compound to organism)
/// This uses the full property-statement-reference pattern to preserve provenance:
/// - p:P703 is the property node
/// - ps:P703 is the statement's object (the taxon)
/// - prov:wasDerivedFrom connects to the reference
/// - pr:P248 is the reference metadata (work/source)
const TAXON_REFERENCE_ASSOCIATION: &str = r#"
  ?c p:P703 ?statement.
  ?statement ps:P703 ?t;
             prov:wasDerivedFrom ?ref.
  ?ref pr:P248 ?r.
  ?t wdt:P225 ?taxon_name.
"#;

/// Reference metadata: title (P1476), DOI (P356), publication date (P577).
/// - P1476: Reference title (optional)
/// - P356: DOI (optional)
/// - P577: Publication date (optional by default; becomes required when year filtering active)
const REFERENCE_METADATA_OPTIONAL: &str = r#"
  OPTIONAL { ?r wdt:P1476 ?ref_title. }
  OPTIONAL { ?r wdt:P356 ?ref_doi. }
  OPTIONAL { ?r wdt:P577 ?ref_date. }
"#;

/// Core variables projected from the innermost (Level-1) SELECT.
/// This is the minimal set needed for the compound–taxon–reference triple lookup;
/// no optional properties are included so QLever can plan the join order freely.
const COMPOUND_CORE_VARS: &str =
    "?c ?compound_inchikey ?compound_smiles_conn ?t ?taxon_name ?r ?ref ?statement";

/// Full variable list projected by the middle (Level-2) SELECT after optional enrichment.
/// Every variable that the outer SELECT clause or any downstream query wrapper may
/// reference must be listed here so QLever can propagate it outward.
const COMPOUND_ENRICHED_VARS: &str = r#"?c ?compound_inchikey ?compound_smiles_conn
      ?compound_smiles_iso ?compound_mass ?compound_formula_raw
      ?compoundLabel
      ?t ?taxon_name
      ?r ?ref
      ?ref_title ?ref_doi ?ref_date
      ?statement"#;

/// Compound properties with efficient subscript digit normalization.
/// - P2017: SMILES isomeric (preferred over P233 when available)
/// - P2067: Molecular mass (optional by default; becomes required when mass filtering active)
/// - P274: Chemical formula (raw fetch; normalization happens at the display/export layer)
/// - rdfs:label in "mul" (multilingual) or "en" (English) language tags
const PROPERTIES_OPTIONAL: &str = r#"
  OPTIONAL { ?c wdt:P2017 ?compound_smiles_iso. }
  OPTIONAL { ?c wdt:P2067 ?compound_mass. }
  OPTIONAL { ?c wdt:P274 ?compound_formula_raw. }
  OPTIONAL { ?c rdfs:label ?compoundLabelMul. FILTER(LANG(?compoundLabelMul) = "mul") }
  OPTIONAL { ?c rdfs:label ?compoundLabelEn. FILTER(LANG(?compoundLabelEn) = "en") }
  BIND(COALESCE(?compoundLabelMul, ?compoundLabelEn) AS ?compoundLabel)
"#;

fn compound_formula_expr(raw_var: &str) -> String {
    normalize_digits_expr(raw_var)
}

fn compound_select_clause() -> String {
    format!(
        r#"
SELECT
  (xsd:integer(STRAFTER(STR(?c), "Q")) AS ?compound)
  ?compoundLabel
  ?compound_inchikey
  ?compound_smiles_conn
  ?compound_smiles_iso
  ?compound_mass
  ({formula} AS ?compound_formula)
  (xsd:integer(STRAFTER(STR(?t), "Q")) AS ?taxon)
  ?taxon_name
  (xsd:integer(STRAFTER(STR(?r), "Q")) AS ?ref_qid)
  ?ref
  ?ref_title
  ?ref_doi
  ?ref_date
  ?statement
"#,
        formula = compound_formula_expr("?compound_formula_raw")
    )
}

/// Search for taxa by scientific name.
///
/// **Method:**
/// Uses Wikidata's P225 (taxon name, scientific nomenclature).
/// Returns all matching Wikidata entities where the scientific name equals the query.
///
/// **Use Cases:**
/// - Autocomplete/suggestions for taxon filtering
/// - Validation that a taxon exists before querying compounds
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

/// Query compounds found in a specific taxon and all descendants.
///
/// **Three-Level Query Structure:**
/// ```text
/// SELECT (outer xsd:integer projections)
/// WHERE {
///   { ── Level 2: middle SELECT ──────────────────────────────────────
///     SELECT COMPOUND_ENRICHED_VARS
///     WHERE {
///       { ── Level 1: innermost SELECT ──────────────────────────────
///         SELECT COMPOUND_CORE_VARS
///         WHERE {
///           core compound-taxon-reference triples
///           ?t (wdt:P171*) wd:Q…  ← ancestry filter INSIDE here
///         }
///       }
///       OPTIONAL { … }   ← reference + property OPTIONALs here
///     }
///   }
/// }
/// ```
///
/// **Why three levels?**
/// - Level 1 applies the `P171*` transitive-closure filter *before* any join,
///   so QLever sees only rows in the target clade when planning OPTIONALs.
/// - Level 2 runs all OPTIONALs exclusively on the already-filtered rows,
///   avoiding expensive enrichment of taxa that are later discarded.
/// - The outer SELECT handles the `xsd:integer(STRAFTER(…))` projections on
///   a tiny, pre-enriched result set.
pub fn query_compounds_by_taxon(taxon_qid: &str) -> String {
    let compound_select = compound_select_clause();
    format!(
        r#"{PREFIXES}
{compound_select}
WHERE {{
  {{
    SELECT
      {COMPOUND_ENRICHED_VARS}
    WHERE {{
      {{
        SELECT {COMPOUND_CORE_VARS}
        WHERE {{
          {COMPOUND_IDENTIFIERS}
          {TAXON_REFERENCE_ASSOCIATION}
          ?t (wdt:P171*) wd:{taxon_qid}.
        }}
      }}
      {REFERENCE_METADATA_OPTIONAL}
      {PROPERTIES_OPTIONAL}
    }}
  }}
}}"#
    )
}

/// Query all compounds from all organisms/taxa in Lotus.
///
/// **Two-Level SELECT Structure (no ancestry filter):**
/// ```text
/// SELECT (outer xsd:integer projections)
/// WHERE {
///   { ── middle SELECT ─────────────────────────────────────────────
///     SELECT COMPOUND_ENRICHED_VARS
///     WHERE {
///       { ── innermost SELECT ─────────────────────────────────────
///         SELECT COMPOUND_CORE_VARS
///         WHERE { core compound-taxon-reference triples }
///       }
///       OPTIONAL { … }   ← OPTIONALs run after inner join completes
///     }
///   }
/// }
/// ```
///
/// Uses the same three-level scaffolding as `query_compounds_by_taxon` to keep
/// optional enrichment strictly post-join, even when no ancestry filter is active.
/// Large result sets should use LIMIT or be paginated via QLever.
pub fn query_all_compounds() -> String {
    let compound_select = compound_select_clause();
    format!(
        r#"{PREFIXES}
{compound_select}
WHERE {{
  {{
    SELECT
      {COMPOUND_ENRICHED_VARS}
    WHERE {{
      {{
        SELECT {COMPOUND_CORE_VARS}
        WHERE {{
          {COMPOUND_IDENTIFIERS}
          {TAXON_REFERENCE_ASSOCIATION}
        }}
      }}
      {REFERENCE_METADATA_OPTIONAL}
      {PROPERTIES_OPTIONAL}
    }}
  }}
}}"#
    )
}

/// Structure similarity/substructure search query via IDSM/Sachem service.
///
/// **Search Types:**
/// - Similarity: Tanimoto similarity filtering (0-1 scale via `threshold`)
/// - Substructure: Finds all compounds containing the query structure
///
/// **Taxon Filtering:**
/// - With taxon: inner SELECT isolates matching compounds, then applies taxon ancestry filter.
/// - Without taxon: structure search proceeds; OPTIONAL enrichment for taxa+refs.
///
/// **Query Optimization:**
/// Sachem service is isolated in a subquery to allow efficient pre-filtering
/// before expensive reference/property lookups. This pattern avoids combinatorial
/// explosions when many compounds match the structure query. When taxon filtering
/// is active, the taxon ancestry filter is applied *inside* the Sachem pass to
/// ensure QLever planner only enriches matching rows.
pub fn query_sachem(
    smiles: &str,
    search_type: SmilesSearchType,
    threshold: f64,
    taxon_qid: Option<&str>,
) -> String {
    let structure_literal = escape_structure_literal(smiles);
    let is_multiline_literal =
        structure_literal.starts_with("'''") || structure_literal.starts_with(r#"\"\"\""#);

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

    let sachem_subquery = format!(
        r#"{{
    SELECT DISTINCT ?c
    WHERE {{
      {sachem_clause}
    }}
  }}"#
    );

    let body = if let Some(qid) = taxon_qid {
        format!(
            r#"
  {sachem_subquery}

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

    let compound_select = compound_select_clause();
    format!(
        r#"{PREFIXES_WITH_STRUCTURE}
{compound_select}
WHERE {{
{body}
}}"#
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureKind {
    Empty,
    Smiles,
    MolfileV2000,
    MolfileV3000,
}

impl StructureKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Empty => "—",
            Self::Smiles => "SMILES",
            Self::MolfileV2000 => "Molfile V2000",
            Self::MolfileV3000 => "Molfile V3000",
        }
    }
}

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

fn looks_like_molfile(text: &str) -> bool {
    matches!(
        classify_structure(text),
        StructureKind::MolfileV2000 | StructureKind::MolfileV3000
    )
}

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

/// Generate a dataset statistics query from a base compound query.
///
/// **Metrics:**
/// - n_entries: total result triples (including duplicates)
/// - n_entries_unique: unique compound-taxon-reference combinations
/// - n_compounds: distinct compounds (Wikidata entities)
/// - n_taxa: distinct organisms
/// - n_references: distinct evidence sources
///
/// **Optimization:**
/// Uses COUNT(DISTINCT ...) to compute cardinality without materializing
/// full result sets. The implementation wraps the base query's WHERE block
/// to preserve all filtering/search logic.
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
  (COUNT(DISTINCT CONCAT(
    STR(?compound), "\u001F", COALESCE(STR(?taxon), ""), "\u001F", COALESCE(STR(?ref_qid), "")
  )) AS ?n_entries_unique)
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

/// Append a LIMIT clause to a base query for pagination or sampling.
///
/// **Use Cases:**
/// - Pagination: fetch first N results, then apply OFFSET for next page
/// - Sampling: LIMIT 100 for quick exploratory queries
/// - UI constraints: avoid overwhelming clients with massive result sets
pub fn query_with_limit(base_query: &str, limit: usize) -> String {
    let trimmed = base_query.trim_end();
    format!("{trimmed}\nLIMIT {limit}")
}

/// Apply server-side filtering conditions (mass, year, molecular formula) to a query.
///
/// **Filter Types:**
/// - Mass: range filter on P2067 (molecular weight)
/// - Year: range filter on P577 (publication date) via YEAR() function
/// - Formula: element count bounds (C/H/N/O/P/S) and halogen requirements (F/Cl/Br/I)
///
/// **Optimization Strategy:**
/// When mass or date filters are present, they are inserted as *required* triples
/// (not OPTIONAL) to ensure the FILTER() clauses only operate on bound rows. This
/// reduces cardinality before optional enrichment and allows QLever to plan joins
/// more efficiently. Filters that don't use mass/date leave those as optional.
///
/// Formula filters use pre-computed element count bindings (REGEX patterns on
/// tokenized formula) to avoid repeated regex evaluation per row.
pub fn query_with_server_filters(base_query: &str, criteria: &SearchCriteria) -> String {
    let mut filters = Vec::new();
    let mut prelude = Vec::new();
    let mut required_inserts = Vec::new();

    if criteria.has_mass_filter() {
        let min = criteria.mass_min;
        let max = criteria.mass_max;
        filters.push(format!(
            "FILTER(?compound_mass >= {min:.6} && ?compound_mass <= {max:.6})"
        ));
        required_inserts.push("?c wdt:P2067 ?compound_mass .".to_string());
    }

    if criteria.has_year_filter() {
        let start = criteria.year_min;
        let end = criteria.year_max;
        filters.push(format!(
            "FILTER(YEAR(?ref_date) >= {start} && YEAR(?ref_date) <= {end})"
        ));
        required_inserts.push("?r wdt:P577 ?ref_date .".to_string());
    }

    if criteria.has_formula_filter() {
        prelude.push("FILTER(BOUND(?compound_formula_raw))".to_string());
        prelude.push("BIND(STR(?compound_formula_raw) AS ?_formula_raw)".to_string());
        prelude.push(r#"BIND(REPLACE(?_formula_raw, " ", "") AS ?_formula_nospace)"#.to_string());
        prelude.push(format!(
            "BIND({} AS ?_formula_norm)",
            normalize_digits_expr("?_formula_nospace")
        ));
        prelude.push(
            "BIND(REPLACE(?_formula_norm, \"([A-Z])\", \"|$1\") AS ?_formula_tokens)".to_string(),
        );

        for (symbol, min, max, default_max) in [
            ("C", criteria.c_min, criteria.c_max, DEFAULT_C_MAX),
            ("H", criteria.h_min, criteria.h_max, DEFAULT_H_MAX),
            ("N", criteria.n_min, criteria.n_max, DEFAULT_N_MAX),
            ("O", criteria.o_min, criteria.o_max, DEFAULT_O_MAX),
            ("P", criteria.p_min, criteria.p_max, DEFAULT_P_MAX),
            ("S", criteria.s_min, criteria.s_max, DEFAULT_S_MAX),
        ] {
            if min > 0 || max < default_max {
                let var = format!("?_count_{}", symbol.to_ascii_lowercase());
                prelude.push(element_count_bind(symbol, &var));
                filters.push(format!("FILTER({var} >= {min} && {var} <= {max})"));
            }
        }

        for (symbol, state) in [
            ("F", criteria.f_state),
            ("Cl", criteria.cl_state),
            ("Br", criteria.br_state),
            ("I", criteria.i_state),
        ] {
            if state != ElementState::Allowed {
                let var = format!("?_count_{}", symbol.to_ascii_lowercase());
                prelude.push(element_count_bind(symbol, &var));
                match state {
                    ElementState::Allowed => {}
                    ElementState::Required => filters.push(format!("FILTER({var} > 0)")),
                    ElementState::Excluded => filters.push(format!("FILTER({var} = 0)")),
                }
            }
        }
    }

    if let Some(exact) = criteria
        .formula_enabled
        .then_some(criteria.formula_exact.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let exact_norm: String = normalize_formula_digits(exact)
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let exact_escaped = exact_norm.replace('\\', r"\\").replace('"', r#"\""#);
        filters.push(format!("FILTER(?_formula_norm = \"{exact_escaped}\")"));
    }

    if prelude.is_empty() && filters.is_empty() && required_inserts.is_empty() {
        return base_query.to_string();
    }

    let trimmed = base_query.trim_end();
    let Some(last_close) = trimmed.rfind('}') else {
        let mut out = String::with_capacity((filters.len() + required_inserts.len()) * 100);
        out.push_str(trimmed);
        for insert in required_inserts {
            out.push('\n');
            out.push_str(&insert);
        }
        for filter in filters {
            out.push('\n');
            out.push_str(&filter);
        }
        return out;
    };

    let mut out = String::with_capacity(
        trimmed.len() + (filters.len() + prelude.len() + required_inserts.len()) * 100,
    );
    out.push_str(&trimmed[..last_close]);
    out.push('\n');

    if !required_inserts.is_empty() {
        for insert in required_inserts {
            out.push_str(&insert);
            out.push('\n');
        }
    }

    if !prelude.is_empty() {
        out.push_str(&prelude.join("\n"));
        out.push('\n');
    }

    out.push_str(&filters.join("\n"));
    out.push('\n');
    out.push_str(&trimmed[last_close..]);
    out
}

/// Transform a SELECT query into a CONSTRUCT query for RDF export.
///
/// **Output Format:**
/// Turtle-serializable RDF triples representing the full compound-taxon-reference
/// relationship graph. This is useful for:
/// - Semantic web integration (LOD/linked data compatible)
/// - Downstream Semantic Web applications
/// - Preserving full statement structure and provenance
///
/// **Pattern:**
/// Maps the SELECT variables to RDF triples using Wikidata vocabulary:
/// compound properties (P235, P233, etc.), taxon info, references, and metadata.
pub fn query_construct_from_select(select_query: &str) -> String {
    let Some(select_pos) = select_query.find("SELECT") else {
        return select_query.to_string();
    };
    let Some(where_pos) = select_query[select_pos..].find("WHERE") else {
        return select_query.to_string();
    };
    let where_abs = select_pos + where_pos;
    let prefixes = &select_query[..select_pos];
    let where_block = select_query[where_abs..].trim();
    let normalized_where_block = construct_where_with_formula_bind(where_block);

    format!(
        r#"{prefixes}
CONSTRUCT {{
  ?c wdt:P235 ?compound_inchikey .
  ?c wdt:P233 ?compound_smiles_conn .
  ?c wdt:P2017 ?compound_smiles_iso .
  ?c wdt:P2067 ?compound_mass .
  ?c wdt:P274 ?compound_formula .
  ?c rdfs:label ?compoundLabel .
  ?c p:P703 ?statement .
  ?statement ps:P703 ?t ;
             prov:wasDerivedFrom ?ref .
  ?ref pr:P248 ?r .
  ?t wdt:P225 ?taxon_name .
  ?r wdt:P1476 ?ref_title .
  ?r wdt:P356 ?ref_doi .
  ?r wdt:P577 ?ref_date .
}}
{where_block}"#,
        prefixes = prefixes,
        where_block = normalized_where_block
    )
}

fn construct_where_with_formula_bind(where_block: &str) -> String {
    let Some(open_brace) = where_block.find('{') else {
        return where_block.to_string();
    };
    let Some(close_brace) = where_block.rfind('}') else {
        return where_block.to_string();
    };
    if close_brace <= open_brace {
        return where_block.to_string();
    }

    let inner = &where_block[(open_brace + 1)..close_brace];
    let formula_bind = format!(
        "  BIND({} AS ?compound_formula)",
        compound_formula_expr("?compound_formula_raw")
    );

    let mut out = String::with_capacity(where_block.len() + formula_bind.len() + 16);
    out.push_str("WHERE {");
    out.push_str(inner);
    out.push('\n');
    out.push_str(&formula_bind);
    out.push_str("\n}");
    out
}

/// Build a BIND expression that normalizes subscript digits (₀-₉) to ASCII (0-9).
/// This handles Wikidata's use of subscript digits in chemical formulas.
/// The expression is left to right, preserving semantic correctness.
fn normalize_digits_expr(var: &str) -> String {
    // Build: BIND(REPLACE(REPLACE(...REPLACE(var, "₀", "0")..., "₉", "9") AS ?result)
    // This is more efficient than deeply nested SELECTs or multiple BINDs.
    SUBSCRIPT_DIGIT_MAPPINGS.iter().fold(
        format!("STR({var})"),
        |acc, &(subscript_char, ascii_digit)| {
            format!(r#"REPLACE({acc}, "{subscript_char}", "{ascii_digit}")"#)
        },
    )
}

fn element_count_bind(symbol: &str, out_var: &str) -> String {
    let escaped = symbol.replace('"', "\\\"");
    let pattern = format!(r#"\\|{escaped}([0-9]*)(\\||$)"#);
    let capture_expr = format!(r#"REPLACE(?_formula_tokens, ".*{pattern}.*", "$1")"#);
    format!(
        "BIND(IF(REGEX(?_formula_tokens, \"{pattern}\"), IF(STRLEN({capture_expr}) = 0, 1, xsd:integer({capture_expr})), 0) AS {out_var})"
    )
}

fn normalize_formula_digits(s: &str) -> String {
    s.chars().map(normalize_formula_digit).collect()
}

fn normalize_formula_digit(c: char) -> char {
    SUBSCRIPT_DIGIT_MAPPINGS
        .iter()
        .find_map(|(from, to)| (*from == c).then_some(*to))
        .unwrap_or(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_filter_query_includes_formula_and_halogen_clauses() {
        let mut crit = SearchCriteria {
            taxon: "*".into(),
            ..SearchCriteria::default()
        };
        crit.formula_enabled = true;
        crit.c_min = 1;
        crit.c_max = 10;
        crit.f_state = ElementState::Required;

        let q = query_with_server_filters(&query_all_compounds(), &crit);
        assert!(q.contains("?_formula_tokens"));
        assert!(q.contains("?_count_c >= 1 && ?_count_c <= 10"));
        assert!(q.contains("?_count_f > 0"));
    }

    #[test]
    fn server_filter_inserts_required_mass_when_mass_filtering() {
        let mut crit = SearchCriteria {
            ..SearchCriteria::default()
        };
        crit.mass_min = 100.0;
        crit.mass_max = 500.0;

        let q = query_with_server_filters(&query_all_compounds(), &crit);
        assert!(q.contains("?c wdt:P2067 ?compound_mass"));
        assert!(q.contains("FILTER(?compound_mass >= 100"));
        assert!(q.contains("?compound_mass <= 500"));
    }

    #[test]
    fn server_filter_inserts_required_date_when_year_filtering() {
        let mut crit = SearchCriteria {
            ..SearchCriteria::default()
        };
        crit.year_min = 2000;
        crit.year_max = 2024;

        let q = query_with_server_filters(&query_all_compounds(), &crit);
        assert!(q.contains("?r wdt:P577 ?ref_date"));
        assert!(q.contains("FILTER(YEAR(?ref_date) >= 2000"));
        assert!(q.contains("YEAR(?ref_date) <= 2024)"));
    }

    #[test]
    fn construct_query_switches_select_to_construct() {
        let q = query_construct_from_select(&query_compounds_by_taxon("Q2382443"));
        assert!(q.contains("CONSTRUCT"));
        assert!(q.contains("?c p:P703 ?statement"));
        assert!(!q.contains("SELECT\n  (xsd:integer"));
    }

    #[test]
    fn sachem_query_uses_combined_prefixes() {
        let q = query_sachem("c1ccccc1", SmilesSearchType::Substructure, 0.8, None);
        // Should have all standard prefixes
        assert!(q.contains("PREFIX xsd:"));
        assert!(q.contains("PREFIX wd:"));
        assert!(q.contains("PREFIX wdt:"));
        // Should have structure-specific prefixes
        assert!(q.contains("PREFIX sachem:"));
        assert!(q.contains("PREFIX idsm:"));
        // Should NOT duplicate prefixes
        assert_eq!(q.matches("PREFIX xsd:").count(), 1);
        assert_eq!(q.matches("PREFIX sachem:").count(), 1);
    }

    #[test]
    fn sachem_taxon_query_applies_ancestry_filter() {
        let q = query_sachem(
            "c1ccccc1",
            SmilesSearchType::Substructure,
            0.8,
            Some("Q158572"),
        );
        assert!(q.contains("?t (wdt:P171*) wd:Q158572"));
        // Should bind taxon_name and reference metadata
        assert!(q.contains("?t wdt:P225 ?taxon_name"));
        assert!(q.contains("?ref pr:P248 ?r"));
    }

    #[test]
    fn sachem_no_taxon_query_makes_taxa_optional() {
        let q = query_sachem("c1ccccc1", SmilesSearchType::Substructure, 0.8, None);
        // Taxa block should be OPTIONAL when no taxon specified
        assert!(q.contains("OPTIONAL {"));
        assert!(q.contains("?c p:P703 ?statement"));
        // Should not have ancestry filter
        assert!(!q.contains("?t (wdt:P171*)"));
    }

    #[test]
    fn count_query_uses_distinct_entry_triples_not_raw_rows() {
        let q = query_counts_from_base(&query_sachem(
            "CCO",
            SmilesSearchType::Substructure,
            0.8,
            Some("Q158572"),
        ));
        assert!(q.contains("COUNT(*) AS ?n_entries"));
        assert!(q.contains("COUNT(DISTINCT CONCAT("));
        assert!(q.contains("AS ?n_entries_unique"));
        assert!(q.contains("STR(?compound)"));
        assert!(q.contains("COALESCE(STR(?taxon), \"\")"));
        assert!(q.contains("COALESCE(STR(?ref_qid), \"\")"));
    }

    #[test]
    fn subscript_digit_normalizers_stay_in_sync() {
        assert_eq!(normalize_formula_digits("C₆H₁₂O₆"), "C6H12O6");

        let expr = normalize_digits_expr("?_formula_nospace");
        for (from, to) in SUBSCRIPT_DIGIT_MAPPINGS {
            assert!(expr.contains(&format!("\"{from}\"")));
            assert!(expr.contains(&format!("\"{to}\"")));
        }
    }

    #[test]
    fn compound_queries_keep_ref_uri_for_rdf_construct_compat() {
        let q = query_all_compounds();
        assert!(q.contains("\n  ?ref\n"));
        assert!(q.contains("?ref_qid"));
    }

    #[test]
    fn compound_queries_project_raw_formula_and_normalize_at_display_layer() {
        let q = query_compounds_by_taxon("Q2382443");
        assert!(q.contains("?compound_formula_raw"));
        assert!(q.contains("AS ?compound_formula"));
    }

    #[test]
    fn server_filters_bind_formula_from_raw_formula_column() {
        let crit = SearchCriteria {
            formula_enabled: true,
            formula_exact: "C6H12O6".into(),
            ..SearchCriteria::default()
        };
        let q = query_with_server_filters(&query_all_compounds(), &crit);
        assert!(q.contains("BOUND(?compound_formula_raw)"));
        assert!(q.contains("STR(?compound_formula_raw)"));
        assert!(q.contains("?_formula_norm"));
    }

    #[test]
    fn construct_query_rebinds_formula_from_raw_column() {
        let q = query_construct_from_select(&query_compounds_by_taxon("Q2382443"));
        assert!(q.contains("?compound_formula_raw"));
        assert!(q.contains("AS ?compound_formula"));
        assert!(q.contains("?c wdt:P274 ?compound_formula ."));
        assert_eq!(q.matches("PREFIX xsd:").count(), 1);
        assert_eq!(q.matches("PREFIX wdt:").count(), 1);
    }

    #[test]
    fn construct_query_normalizes_formula_subscript_digits() {
        let q = query_construct_from_select(&query_compounds_by_taxon("Q2382443"));
        assert!(q.contains("BIND("));
        assert!(q.contains("STR(?compound_formula_raw)"));
        // Regression guard: keep subscript-digit normalization in RDF export.
        assert!(q.contains("\"₆\""));
        assert!(q.contains("\"6\""));
    }

    #[test]
    fn sachem_query_projects_formula_from_raw_column() {
        let q = query_sachem("c1ccccc1", SmilesSearchType::Substructure, 0.8, None);
        // The formula column must be derived from ?compound_formula_raw, not a bare ?compound_formula
        assert!(q.contains("?compound_formula_raw"));
        assert!(q.contains("AS ?compound_formula"));
    }

    #[test]
    fn prefixes_once_per_query_not_duplicated() {
        let q1 = query_all_compounds();
        assert_eq!(q1.matches("PREFIX xsd:").count(), 1);
        assert_eq!(q1.matches("PREFIX rdfs:").count(), 1);
        assert_eq!(q1.matches("PREFIX wd:").count(), 1);

        let q2 = query_compounds_by_taxon("Q2382443");
        assert_eq!(q2.matches("PREFIX xsd:").count(), 1);
        assert_eq!(q2.matches("PREFIX wdt:").count(), 1);

        let q3 = query_sachem("c1ccccc1", SmilesSearchType::Substructure, 0.8, None);
        assert_eq!(q3.matches("PREFIX sachem:").count(), 1);
        assert_eq!(q3.matches("PREFIX idsm:").count(), 1);
    }
}
