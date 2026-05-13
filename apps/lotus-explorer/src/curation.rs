// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::i18n::{
    Locale, curation_note_dependencies_pending, curation_note_existing_complete,
    curation_note_existing_updates, curation_note_new_compound, curation_pending_reference,
    curation_pending_taxon,
};
use crate::sparql::execute_sparql_format;
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shared::sparql::SparqlResponseFormat;
use std::collections::HashSet;
use std::fmt;

// ── Sub-modules ───────────────────────────────────────────────────────────────

#[path = "curation/helpers.rs"]
mod helpers;
use helpers::*;

#[path = "curation/http_client.rs"]
mod http_client;
use http_client::*;

#[path = "curation/chemical.rs"]
mod chemical;
use chemical::*;

#[path = "curation/wikidata.rs"]
mod wikidata;
use wikidata::*;

#[path = "curation/reference_metadata.rs"]
mod reference_metadata;
use reference_metadata::*;

#[path = "curation/share_links.rs"]
mod share_links;
#[cfg(test)]
use share_links::{CURATION_ROWS_PARAM, curation_rows_from_query_params};
pub use share_links::{
    build_curation_share_url, initial_curation_autorun_from_url, initial_curation_rows_from_url,
};

// ── Constants ─────────────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
const NATPROD_API_BASE: &str = "https://api.naturalproducts.net/latest";

const CURATION_SPARQL_PREFIXES: &str = "\
PREFIX wd: <http://www.wikidata.org/entity/>\n\
PREFIX wdt: <http://www.wikidata.org/prop/direct/>\n\
PREFIX p: <http://www.wikidata.org/prop/>\n\
PREFIX ps: <http://www.wikidata.org/prop/statement/>\n\
PREFIX prov: <http://www.w3.org/ns/prov#>\n\
PREFIX pr: <http://www.wikidata.org/prop/reference/>\n\
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>";

const WD_CHEMICAL_COMPOUND_QID: &str = "Q11173";
const WD_TYPE_CHEMICAL_ENTITY_QID: &str = "Q113145171";
const WD_STEREOISOMER_GROUP_QID: &str = "Q59199015";
const WD_OCCURS_IN_TAXON_PROP: &str = "P703";
const WD_TAXON_QID: &str = "Q16521";

// ──────────────────────────────────────────────────────────────────────────────
// Public types
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurationInputRow {
    pub name: String,
    pub smiles: String,
    pub taxon: Option<String>,
    pub doi: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CurationStatus {
    ExistingComplete,
    ExistingNeedsUpdates,
    NewCompound,
    PendingDependencies,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CurationResultRow {
    pub input: CurationInputRow,
    pub canonical_smiles: Option<String>,
    pub inchikey: Option<String>,
    pub inchi: Option<String>,
    pub formula: Option<String>,
    pub exact_mass: Option<f64>,
    pub mass_warning: Option<String>,
    pub wikidata_qid: Option<String>,
    pub status: CurationStatus,
    pub note: String,
    pub dependency_blocks: Vec<String>,
    pub quickstatements: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickStatementsBundle {
    pub dependencies: std::sync::Arc<str>,
    pub main: std::sync::Arc<str>,
}

impl Default for QuickStatementsBundle {
    fn default() -> Self {
        Self {
            dependencies: std::sync::Arc::<str>::from(""),
            main: std::sync::Arc::<str>::from(""),
        }
    }
}

#[derive(Debug)]
pub enum CurationError {
    InvalidInput(String),
    Http(String),
    Parse(String),
}

impl fmt::Display for CurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(msg) => write!(f, "{msg}"),
            Self::Http(msg) => write!(f, "{msg}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Internal types
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct WikidataCompound {
    qid: String,
    canonical_smiles: Option<String>,
    isomeric_smiles: Option<String>,
    inchi: Option<String>,
    formula: Option<String>,
    mass: Option<f64>,
}

#[derive(Debug, Default)]
struct DependencyResolution {
    taxon_qid: Option<String>,
    reference_qid: Option<String>,
    dependency_blocks: Vec<String>,
    pending_messages: Vec<String>,
}

#[derive(Debug, Default)]
struct MassResolution {
    exact_mass: Option<f64>,
    warning: Option<String>,
}

const CURATION_CONCURRENCY: usize = 8;

// ──────────────────────────────────────────────────────────────────────────────
// Example data
// ──────────────────────────────────────────────────────────────────────────────

pub fn example_rows() -> Vec<CurationInputRow> {
    vec![
        CurationInputRow {
            name: "Voatriafricanine A".to_string(),
            smiles: "OC12N3C4=C(O)C([C@H](C[C@H]/5[C@@H]6C(OC)=O)C(N([H])C7=C8C=CC=C7)=C8CC6N(C)CC5=C\\C)=CC=C4[C@@]19CCN%10C9[C@@]%11(C[C@H]2C[C@H]%12[C@H]%13[C@@]%14(CC(C(OC)=O)=C%15NC%16=CC=CC=C%16[C@@]%15%17CCN([C@@H]%123)C%14%17)CCO%13)CCO[C@H]%11CC%10".to_string(),
            taxon: Some("Voacanga africana".to_string()),
            doi: Some("10.1021/acs.jnatprod.1c00812".to_string()),
        },
        CurationInputRow {
            name: "Voatriafricanine B (taxon and DOI wrong but new)".to_string(),
            smiles: "OC12N3C4=C(O)C([C@H](C[C@H]/5[C@@H]6C(OC)=O)C(N([H])C7=C8C=CC=C7)=C8CC6N(C)CC5=C\\C)=CC=C4[C@@]19CCN%10C9[C@@]%11(C[C@H]2C[C@H]%12[C@H]%13[C@@]%14(CC(C(OC)=O)=C%15NC%16=C(OC)C=CC=C%16[C@@]%15%17CCN([C@@H]%123)C%14%17)CCO%13)CCO[C@H]%11CC%10".to_string(),
            taxon: Some("Gentiana lutea".to_string()),
            doi: Some("10.1068/P080363".to_string()),
        },
        CurationInputRow {
            name: "[HYPOTHETICAL — non-real test case]".to_string(),
            smiles: "CCN(CC)C(=O)N1C=NC2=C1N=CN2C(F)(F)F".to_string(),
            taxon: Some("Ficticia imaginaria".to_string()),
            doi: Some("10.59350/sk00y-3gh44".to_string()),
        },
    ]
}

// ──────────────────────────────────────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────────────────────────────────────

pub fn parse_tsv_rows(tsv: &str) -> Result<Vec<CurationInputRow>, CurationError> {
    let mut lines = tsv
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        return Ok(Vec::new());
    }

    let header = lines.remove(0);
    let columns = header.split('\t').map(normalize_header).collect::<Vec<_>>();
    let name_idx = columns
        .iter()
        .position(|c| c == "name")
        .ok_or_else(|| CurationError::InvalidInput("TSV is missing a 'name' column".to_string()))?;
    let smiles_idx = columns.iter().position(|c| c == "smiles").ok_or_else(|| {
        CurationError::InvalidInput("TSV is missing a 'smiles' column".to_string())
    })?;
    let taxon_idx = columns
        .iter()
        .position(|c| matches!(c.as_str(), "taxon" | "organism"));
    let doi_idx = columns.iter().position(|c| c == "doi");

    let mut out = Vec::new();
    for line in lines {
        let fields = line.split('\t').map(str::trim).collect::<Vec<_>>();
        let Some(name) = fields.get(name_idx) else {
            continue;
        };
        let Some(smiles) = fields.get(smiles_idx) else {
            continue;
        };
        if name.is_empty() || smiles.is_empty() {
            continue;
        }
        let taxon = taxon_idx
            .and_then(|idx| fields.get(idx))
            .and_then(|v| non_empty(v).map(ToOwned::to_owned));
        let doi = doi_idx
            .and_then(|idx| fields.get(idx))
            .and_then(|value| normalize_doi(value));
        out.push(CurationInputRow {
            name: (*name).to_string(),
            smiles: (*smiles).to_string(),
            taxon,
            doi,
        });
    }
    Ok(out)
}

pub async fn curate_rows(
    locale: Locale,
    rows: Vec<CurationInputRow>,
) -> Result<(Vec<CurationResultRow>, QuickStatementsBundle), CurationError> {
    let mut seen_keys = HashSet::new();
    let mut unique_rows = Vec::with_capacity(rows.len());
    for row in rows {
        if seen_keys.insert(row_uniqueness_key(&row)) {
            unique_rows.push(row);
        }
    }

    let mut indexed_results = stream::iter(unique_rows.into_iter().enumerate())
        .map(|(idx, row)| async move { (idx, curate_single_row(locale, row).await) })
        .buffer_unordered(CURATION_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;
    indexed_results.sort_by_key(|(idx, _)| *idx);
    let results = indexed_results
        .into_iter()
        .map(|(_, row)| row)
        .collect::<Vec<_>>();

    let mut seen_dependency_blocks = HashSet::new();
    let dependencies = results
        .iter()
        .flat_map(|r| r.dependency_blocks.iter())
        .filter(|block| !block.trim().is_empty())
        .filter(|block| seen_dependency_blocks.insert((*block).clone()))
        .cloned()
        .collect::<Vec<_>>()
        .join("\n\n");

    let main = results
        .iter()
        .filter(|r| !r.quickstatements.is_empty())
        .map(|r| r.quickstatements.join("\n"))
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok((
        results,
        QuickStatementsBundle {
            dependencies: std::sync::Arc::<str>::from(dependencies),
            main: std::sync::Arc::<str>::from(main),
        },
    ))
}

pub fn build_quickstatements_bundle(results: &[CurationResultRow]) -> QuickStatementsBundle {
    let mut seen_dependency_blocks = HashSet::new();
    let dependencies = results
        .iter()
        .flat_map(|r| r.dependency_blocks.iter())
        .filter(|block| !block.trim().is_empty())
        .filter(|block| seen_dependency_blocks.insert((*block).clone()))
        .cloned()
        .collect::<Vec<_>>()
        .join("\n\n");

    let main = results
        .iter()
        .filter(|r| !r.quickstatements.is_empty())
        .map(|r| r.quickstatements.join("\n"))
        .collect::<Vec<_>>()
        .join("\n\n");

    QuickStatementsBundle {
        dependencies: std::sync::Arc::<str>::from(dependencies),
        main: std::sync::Arc::<str>::from(main),
    }
}

pub fn row_uniqueness_key(row: &CurationInputRow) -> String {
    let smiles = row.smiles.trim();
    let taxon = row
        .taxon
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_ascii_lowercase())
        .unwrap_or_default();
    let doi = row
        .doi
        .as_deref()
        .and_then(normalize_doi)
        .unwrap_or_default();
    format!("{smiles}\t{taxon}\t{doi}")
}

// ──────────────────────────────────────────────────────────────────────────────
// Core curation logic
// ──────────────────────────────────────────────────────────────────────────────

async fn curate_single_row(locale: Locale, input: CurationInputRow) -> CurationResultRow {
    match enrich_and_generate(locale, &input).await {
        Ok(result) => result,
        Err(err) => CurationResultRow {
            input,
            canonical_smiles: None,
            inchikey: None,
            inchi: None,
            formula: None,
            exact_mass: None,
            mass_warning: None,
            wikidata_qid: None,
            status: CurationStatus::Error,
            note: err.to_string(),
            dependency_blocks: Vec::new(),
            quickstatements: Vec::new(),
        },
    }
}

async fn enrich_and_generate(
    locale: Locale,
    input: &CurationInputRow,
) -> Result<CurationResultRow, CurationError> {
    let converted = convert_smiles(&input.smiles).await?;
    let mass_resolution = resolve_exact_mass(&input.smiles, &converted.canonical_smiles).await;
    let exact_mass = mass_resolution.exact_mass;
    let formula_from_inchi =
        extract_formula_from_inchi(&converted.inchi).map(|f| normalize_formula_for_wikidata(&f));
    let normalized_doi = input.doi.as_deref().and_then(normalize_doi);
    let wd_compound = fetch_wikidata_compound_by_inchikey(&converted.inchikey).await?;

    let result = match wd_compound {
        Some(existing) => {
            let mut lines: Vec<String> = Vec::new();
            let mut changes = 0usize;
            let mut status = CurationStatus::ExistingComplete;
            let mut note = curation_note_existing_complete(locale).to_string();

            if existing.canonical_smiles.is_none() {
                lines.push(format!(
                    "{}|P233|\"{}\"",
                    existing.qid,
                    escape_qs_string(&converted.canonical_smiles)
                ));
                changes += 1;
            }
            if existing.inchi.is_none() {
                lines.push(format!(
                    "{}|P234|\"{}\"",
                    existing.qid,
                    escape_qs_string(&converted.inchi)
                ));
                changes += 1;
            }
            if existing.formula.is_none() && formula_from_inchi.is_some() {
                lines.push(format!(
                    "{}|P274|\"{}\"",
                    existing.qid,
                    escape_qs_string(formula_from_inchi.as_deref().unwrap_or_default())
                ));
                changes += 1;
            }
            if existing.mass.is_none() && exact_mass.is_some() {
                lines.push(qs_mass_statement(
                    &existing.qid,
                    exact_mass.unwrap_or_default(),
                ));
                changes += 1;
            }
            if has_isomeric_smiles(&converted.isomeric_smiles) && existing.isomeric_smiles.is_none()
            {
                lines.push(format!(
                    "{}|P2017|\"{}\"",
                    existing.qid,
                    escape_qs_string(&converted.isomeric_smiles)
                ));
                changes += 1;
            }

            let dependencies =
                resolve_row_dependencies(locale, input, normalized_doi.as_deref()).await?;

            if let Some(deps) = dependencies.as_ref() {
                let (should_add, p703) = match (
                    deps.taxon_qid.as_deref(),
                    deps.reference_qid.as_deref(),
                    normalized_doi.as_deref(),
                ) {
                    (Some(tqid), Some(rqid), _) => {
                        let add = !compound_has_taxon_with_ref(&existing.qid, tqid, rqid).await?;
                        (
                            add,
                            format!(
                                "{}|{}|{}|S248|{}",
                                existing.qid, WD_OCCURS_IN_TAXON_PROP, tqid, rqid
                            ),
                        )
                    }
                    // Reference item does not exist yet: generate it in dependencies, then rerun.
                    (Some(_), None, Some(_)) => (false, String::new()),
                    (Some(tqid), None, None) => {
                        let add = !compound_has_taxon(&existing.qid, tqid).await?;
                        (
                            add,
                            format!("{}|{}|{}", existing.qid, WD_OCCURS_IN_TAXON_PROP, tqid),
                        )
                    }
                    // Existing compound cannot point to LAST from dependency block safely.
                    (None, _, _) => (false, String::new()),
                };

                if should_add {
                    lines.push(p703);
                    changes += 1;
                }

                if !deps.pending_messages.is_empty() {
                    status = CurationStatus::PendingDependencies;
                    note = format!(
                        "{} {}",
                        curation_note_dependencies_pending(locale),
                        deps.pending_messages.join(" ")
                    );
                }
            }

            if matches!(status, CurationStatus::ExistingComplete) {
                if changes == 0 {
                    note = curation_note_existing_complete(locale).to_string();
                } else {
                    status = CurationStatus::ExistingNeedsUpdates;
                    note = curation_note_existing_updates(locale).to_string();
                }
            }

            CurationResultRow {
                input: input.clone(),
                canonical_smiles: Some(converted.canonical_smiles),
                inchikey: Some(converted.inchikey),
                inchi: Some(converted.inchi),
                formula: existing.formula.or(formula_from_inchi),
                exact_mass: existing.mass.or(exact_mass),
                mass_warning: if existing.mass.is_some() {
                    None
                } else {
                    mass_resolution.warning
                },
                wikidata_qid: Some(existing.qid.clone()),
                status,
                note,
                dependency_blocks: dependencies
                    .map(|deps| deps.dependency_blocks)
                    .unwrap_or_default(),
                quickstatements: lines,
            }
        }
        None => {
            let dependencies =
                resolve_row_dependencies(locale, input, normalized_doi.as_deref()).await?;
            let mut lines = vec!["CREATE".to_string()];
            lines.push(format!("LAST|Len|\"{}\"", escape_qs_string(&input.name)));
            lines.push("LAST|Den|\"chemical compound\"".to_string());

            if has_undefined_stereo(&input.smiles).await {
                lines.push(format!("LAST|P31|{WD_STEREOISOMER_GROUP_QID}"));
            } else {
                lines.push(format!("LAST|P31|{WD_TYPE_CHEMICAL_ENTITY_QID}"));
            }
            lines.push(format!("LAST|P279|{WD_CHEMICAL_COMPOUND_QID}"));
            lines.push(format!(
                "LAST|P235|\"{}\"",
                escape_qs_string(&converted.inchikey)
            ));
            lines.push(format!(
                "LAST|P233|\"{}\"",
                escape_qs_string(&converted.canonical_smiles)
            ));
            if has_isomeric_smiles(&converted.isomeric_smiles) {
                lines.push(format!(
                    "LAST|P2017|\"{}\"",
                    escape_qs_string(&converted.isomeric_smiles)
                ));
            }
            lines.push(format!(
                "LAST|P234|\"{}\"",
                escape_qs_string(&converted.inchi)
            ));
            if let Some(formula) = formula_from_inchi.as_deref() {
                lines.push(format!("LAST|P274|\"{}\"", escape_qs_string(formula)));
            }
            if let Some(mass) = exact_mass {
                lines.push(qs_mass_statement("LAST", mass));
            }

            if let Some(deps) = dependencies.as_ref() {
                let p703 = match (
                    deps.taxon_qid.as_deref(),
                    deps.reference_qid.as_deref(),
                    normalized_doi.as_deref(),
                ) {
                    (Some(tqid), Some(rqid), _) => {
                        format!("LAST|{}|{}|S248|{}", WD_OCCURS_IN_TAXON_PROP, tqid, rqid)
                    }
                    // Reference item does not exist yet: create in dependency block and rerun.
                    (Some(_), None, Some(_)) => String::new(),
                    (Some(tqid), None, None) => {
                        format!("LAST|{}|{}", WD_OCCURS_IN_TAXON_PROP, tqid)
                    }
                    (None, _, _) => {
                        format!("LAST|{}|[NEW_TAXON_QID]", WD_OCCURS_IN_TAXON_PROP)
                    }
                };
                if !p703.is_empty() {
                    lines.push(p703);
                }
            }

            let (status, note) = if let Some(deps) = dependencies.as_ref() {
                if deps.pending_messages.is_empty() {
                    (
                        CurationStatus::NewCompound,
                        curation_note_new_compound(locale).to_string(),
                    )
                } else {
                    (
                        CurationStatus::PendingDependencies,
                        format!(
                            "{} {}",
                            curation_note_dependencies_pending(locale),
                            deps.pending_messages.join(" ")
                        ),
                    )
                }
            } else {
                (
                    CurationStatus::NewCompound,
                    curation_note_new_compound(locale).to_string(),
                )
            };

            CurationResultRow {
                input: input.clone(),
                canonical_smiles: Some(converted.canonical_smiles),
                inchikey: Some(converted.inchikey),
                inchi: Some(converted.inchi),
                formula: formula_from_inchi,
                exact_mass,
                mass_warning: mass_resolution.warning,
                wikidata_qid: None,
                status,
                note,
                dependency_blocks: dependencies
                    .map(|deps| deps.dependency_blocks)
                    .unwrap_or_default(),
                quickstatements: lines,
            }
        }
    };

    Ok(result)
}

async fn resolve_row_dependencies(
    locale: Locale,
    input: &CurationInputRow,
    normalized_doi: Option<&str>,
) -> Result<Option<DependencyResolution>, CurationError> {
    let Some(taxon_name) = input.taxon.as_deref() else {
        return Ok(None);
    };

    let (taxon_qid_opt, taxon_new_qs) = resolve_or_create_taxon(taxon_name).await?;
    let (ref_qid_opt, ref_new_qs) = if let Some(doi) = normalized_doi {
        resolve_or_create_reference(doi).await?
    } else {
        (None, Vec::new())
    };

    let mut resolution = DependencyResolution {
        taxon_qid: taxon_qid_opt,
        reference_qid: ref_qid_opt,
        ..DependencyResolution::default()
    };

    if !taxon_new_qs.is_empty() {
        resolution.dependency_blocks.push(taxon_new_qs.join("\n"));
    }
    if !ref_new_qs.is_empty() {
        resolution.dependency_blocks.push(ref_new_qs.join("\n"));
    }

    if resolution.taxon_qid.is_none() {
        resolution
            .pending_messages
            .push(curation_pending_taxon(locale, taxon_name));
    }
    if normalized_doi.is_some() && resolution.reference_qid.is_none() {
        resolution.pending_messages.push(curation_pending_reference(
            locale,
            normalized_doi.unwrap_or_default(),
        ));
    }

    Ok(Some(resolution))
}

/// Fetch pre-generated QuickStatements from Scholia (native) or citation.js (WASM)
async fn resolve_or_create_reference(
    doi: &str,
) -> Result<(Option<String>, Vec<String>), CurationError> {
    // Check if reference already exists in Wikidata
    if let Some(qid) = resolve_reference_qid(doi).await? {
        return Ok((Some(qid), Vec::new()));
    }

    // Try to fetch QuickStatements from Scholia or citation.js
    let qs_lines = fetch_reference_quickstatements(doi)
        .await
        .unwrap_or_default();

    Ok((None, qs_lines))
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn parse_tsv_supports_expected_headers() {
        let tsv = "name\tsmiles\torganism\tdoi\nA\tCCO\tTaxon\thttps://doi.org/10.1/x\n";
        let rows = parse_tsv_rows(tsv).expect("tsv parse");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "A");
        assert_eq!(rows[0].smiles, "CCO");
        assert_eq!(rows[0].taxon.as_deref(), Some("Taxon"));
        assert_eq!(rows[0].doi.as_deref(), Some("10.1/X"));
    }

    #[test]
    fn extract_formula_reads_inchi_main_layer() {
        assert_eq!(
            extract_formula_from_inchi("InChI=1S/C8H10N4O2/c1-10-4-9-6-5(10)7(13)12(3)8(14)11(6)2"),
            Some("C8H10N4O2".to_string())
        );
    }

    #[test]
    fn normalize_formula_produces_subscript_digits() {
        assert_eq!(normalize_formula_for_wikidata("C8H10N4O2"), "C₈H₁₀N₄O₂");
    }

    #[test]
    fn normalize_formula_passes_through_non_digit_chars() {
        assert_eq!(normalize_formula_for_wikidata("C10H12F3N5O"), "C₁₀H₁₂F₃N₅O");
    }

    #[test]
    fn row_key_normalizes_taxon_and_doi() {
        let row = CurationInputRow {
            name: "compound A".to_string(),
            smiles: " CCO ".to_string(),
            taxon: Some("  Voacanga africana ".to_string()),
            doi: Some("https://doi.org/10.1000/abc".to_string()),
        };
        assert_eq!(
            row_uniqueness_key(&row),
            "CCO\tvoacanga africana\t10.1000/ABC"
        );
    }

    #[test]
    fn qs_mass_uses_unit_not_qunit() {
        // Unit syntax in QS is U<QID>, not UQ<QID>.
        let stmt = qs_mass_statement("LAST", 495.20268);
        assert!(stmt.contains("U483261"), "expected U483261 but got: {stmt}");
        assert!(
            !stmt.contains("UQ483261"),
            "must not contain UQ483261: {stmt}"
        );
    }

    #[test]
    fn extract_exact_mass_from_nested_json_dict() {
        let payload = serde_json::json!({
            "CCO": {
                "exact_molecular_weight": 46.04186,
                "molecular_formula": "C2H6O"
            }
        });
        assert_eq!(extract_exact_mass_from_json(&payload), Some(46.04186));
    }

    #[test]
    fn extract_exact_mass_from_nested_json_array() {
        let payload = serde_json::json!({
            "results": [
                {"foo": "bar"},
                {"descriptors": {"exact_molecular_weight": 180.06339}}
            ]
        });
        assert_eq!(extract_exact_mass_from_json(&payload), Some(180.06339));
    }

    #[test]
    fn extract_exact_mass_from_string_number_with_grouping() {
        let payload = serde_json::json!({
            "exact_molecular_weight": "1,234.5678"
        });
        assert_eq!(extract_exact_mass_from_json(&payload), Some(1234.5678));
    }

    #[test]
    fn curation_share_params_roundtrip_rows() {
        let rows = vec![
            CurationInputRow {
                name: "Compound A".to_string(),
                smiles: "CCO".to_string(),
                taxon: Some("Gentiana lutea".to_string()),
                doi: Some("10.1000/ABC".to_string()),
            },
            CurationInputRow {
                name: "Compound B".to_string(),
                smiles: "C1=CC=CC=C1".to_string(),
                taxon: None,
                doi: None,
            },
        ];
        let mut params = BTreeMap::new();
        params.insert(
            CURATION_ROWS_PARAM.to_string(),
            serde_json::to_string(&rows).expect("rows json"),
        );
        assert_eq!(curation_rows_from_query_params(&params), rows);
    }

    #[test]
    fn curation_share_url_contains_view_and_autorun() {
        let rows = vec![CurationInputRow {
            name: "Compound A".to_string(),
            smiles: "CCO".to_string(),
            taxon: None,
            doi: None,
        }];
        let url = build_curation_share_url(&rows, Locale::Fr, true).expect("share url");
        assert!(url.contains("view=curation-explorer"));
        assert!(url.contains("lang=fr"));
        assert!(url.contains("curation_run=true"));
        assert!(url.contains("curation_rows="));
    }

    #[test]
    fn build_quickstatements_bundle_deduplicates_dependencies_and_joins_sections() {
        let rows = vec![
            CurationResultRow {
                input: CurationInputRow {
                    name: "A".into(),
                    smiles: "C".into(),
                    taxon: None,
                    doi: None,
                },
                canonical_smiles: None,
                inchikey: None,
                inchi: None,
                formula: None,
                exact_mass: None,
                mass_warning: None,
                wikidata_qid: None,
                status: CurationStatus::NewCompound,
                note: String::new(),
                dependency_blocks: vec!["DEP-1".into(), "DEP-1".into()],
                quickstatements: vec!["MAIN-1A".into(), "MAIN-1B".into()],
            },
            CurationResultRow {
                input: CurationInputRow {
                    name: "B".into(),
                    smiles: "N".into(),
                    taxon: None,
                    doi: None,
                },
                canonical_smiles: None,
                inchikey: None,
                inchi: None,
                formula: None,
                exact_mass: None,
                mass_warning: None,
                wikidata_qid: None,
                status: CurationStatus::NewCompound,
                note: String::new(),
                dependency_blocks: vec!["DEP-1".into(), "DEP-2".into()],
                quickstatements: vec!["MAIN-2".into()],
            },
        ];

        let bundle = build_quickstatements_bundle(&rows);
        assert_eq!(bundle.dependencies.as_ref(), "DEP-1\n\nDEP-2");
        assert_eq!(bundle.main.as_ref(), "MAIN-1A\nMAIN-1B\n\nMAIN-2");
    }
}
