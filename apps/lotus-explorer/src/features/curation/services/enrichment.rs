// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AskCacheKey {
    Taxon {
        compound_qid: String,
        taxon_qid: String,
    },
    TaxonWithRef {
        compound_qid: String,
        taxon_qid: String,
        ref_qid: String,
    },
}

#[derive(Default)]
pub(crate) struct OccurrenceAskCache {
    values: HashMap<AskCacheKey, bool>,
}

fn read_cached_ask(cache: &Mutex<OccurrenceAskCache>, key: &AskCacheKey) -> Option<bool> {
    cache
        .lock()
        .ok()
        .and_then(|guard| guard.values.get(key).copied())
}

fn write_cached_ask(cache: &Mutex<OccurrenceAskCache>, key: AskCacheKey, value: bool) {
    if let Ok(mut guard) = cache.lock() {
        guard.values.insert(key, value);
    }
}

async fn compound_has_taxon_cached(
    cache: &Mutex<OccurrenceAskCache>,
    compound_qid: &str,
    taxon_qid: &str,
) -> Result<bool, CurationError> {
    let key = AskCacheKey::Taxon {
        compound_qid: compound_qid.to_string(),
        taxon_qid: taxon_qid.to_string(),
    };

    if let Some(cached) = read_cached_ask(cache, &key) {
        return Ok(cached);
    }

    let value = compound_has_taxon(compound_qid, taxon_qid).await?;
    write_cached_ask(cache, key, value);
    Ok(value)
}

async fn compound_has_taxon_with_ref_cached(
    cache: &Mutex<OccurrenceAskCache>,
    compound_qid: &str,
    taxon_qid: &str,
    ref_qid: &str,
) -> Result<bool, CurationError> {
    let key = AskCacheKey::TaxonWithRef {
        compound_qid: compound_qid.to_string(),
        taxon_qid: taxon_qid.to_string(),
        ref_qid: ref_qid.to_string(),
    };

    if let Some(cached) = read_cached_ask(cache, &key) {
        return Ok(cached);
    }

    let value = compound_has_taxon_with_ref(compound_qid, taxon_qid, ref_qid).await?;
    write_cached_ask(cache, key, value);
    Ok(value)
}

pub(crate) async fn curate_single_row(
    locale: Locale,
    input: CurationInputRow,
    prefetched_taxa: Arc<HashMap<String, String>>,
    prefetched_references: Arc<HashMap<String, String>>,
    occurrence_ask_cache: Arc<Mutex<OccurrenceAskCache>>,
) -> CurationResultRow {
    match enrich_and_generate(
        locale,
        &input,
        prefetched_taxa.as_ref(),
        prefetched_references.as_ref(),
        occurrence_ask_cache.as_ref(),
    )
    .await
    {
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
    prefetched_taxa: &HashMap<String, String>,
    prefetched_references: &HashMap<String, String>,
    occurrence_ask_cache: &Mutex<OccurrenceAskCache>,
) -> Result<CurationResultRow, CurationError> {
    let converted = convert_smiles(&input.smiles).await?;
    let formula_from_inchi =
        extract_formula_from_inchi(&converted.inchi).map(|f| normalize_formula_for_wikidata(&f));
    let normalized_doi = input.doi.as_deref().and_then(normalize_doi);
    let wd_compound = fetch_wikidata_compound_by_inchikey(&converted.inchikey).await?;

    let result = match wd_compound {
        Some(existing) => {
            let mass_resolution = if existing.mass.is_none() {
                resolve_exact_mass(&input.smiles, &converted.canonical_smiles).await
            } else {
                MassResolution {
                    exact_mass: None,
                    warning: None,
                }
            };
            let exact_mass = mass_resolution.exact_mass;

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

            let dependencies = resolve_row_dependencies(
                locale,
                input,
                normalized_doi.as_deref(),
                prefetched_taxa,
                prefetched_references,
            )
            .await?;

            if let Some(deps) = dependencies.as_ref() {
                let (should_add, p703) = match (
                    deps.taxon_qid.as_deref(),
                    deps.reference_qid.as_deref(),
                    normalized_doi.as_deref(),
                ) {
                    (Some(tqid), Some(rqid), _) => {
                        let add = !compound_has_taxon_with_ref_cached(
                            occurrence_ask_cache,
                            &existing.qid,
                            tqid,
                            rqid,
                        )
                        .await?;
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
                        let add =
                            !compound_has_taxon_cached(occurrence_ask_cache, &existing.qid, tqid)
                                .await?;
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
            let mass_resolution =
                resolve_exact_mass(&input.smiles, &converted.canonical_smiles).await;
            let exact_mass = mass_resolution.exact_mass;

            let dependencies = resolve_row_dependencies(
                locale,
                input,
                normalized_doi.as_deref(),
                prefetched_taxa,
                prefetched_references,
            )
            .await?;
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
    prefetched_taxa: &HashMap<String, String>,
    prefetched_references: &HashMap<String, String>,
) -> Result<Option<DependencyResolution>, CurationError> {
    let Some(taxon_name) = input.taxon.as_deref() else {
        return Ok(None);
    };

    let prefetched_taxon_qid = normalize_taxon_lookup(taxon_name)
        .and_then(|lookup| prefetched_taxa.get(&lookup))
        .map(String::as_str);
    let (taxon_qid_opt, taxon_new_qs) =
        resolve_or_create_taxon(taxon_name, prefetched_taxon_qid).await?;
    let (ref_qid_opt, ref_new_qs) = if let Some(doi) = normalized_doi {
        let prefetched_ref_qid = prefetched_references.get(doi).map(String::as_str);
        resolve_or_create_reference(doi, prefetched_ref_qid).await?
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
    pre_resolved_qid: Option<&str>,
) -> Result<(Option<String>, Vec<String>), CurationError> {
    if let Some(qid) = pre_resolved_qid {
        return Ok((Some(qid.to_string()), Vec::new()));
    }

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
