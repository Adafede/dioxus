// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project
#[cfg(not(target_arch = "wasm32"))]
pub(super) use crate::features::curation::domain::NATPROD_API_BASE;
pub(super) use crate::features::curation::domain::{
    CURATION_SPARQL_PREFIXES, CurationError, CurationInputRow, CurationResultRow, CurationStatus,
    DependencyResolution, MassResolution, WD_CHEMICAL_COMPOUND_QID, WD_OCCURS_IN_TAXON_PROP,
    WD_STEREOISOMER_GROUP_QID, WD_TAXON_QID, WD_TYPE_CHEMICAL_ENTITY_QID, WikidataCompound,
};
use crate::i18n::{
    Locale, curation_note_dependencies_pending, curation_note_existing_complete,
    curation_note_existing_updates, curation_note_new_compound, curation_pending_reference,
    curation_pending_taxon,
};
use crate::sparql::execute_sparql_format;
use serde::Deserialize;
use serde_json::Value;
use shared::sparql::SparqlResponseFormat;

mod chemical;
mod enrichment;
mod helpers;
mod http_client;
mod occurrence_cache;
mod reference_metadata;
pub mod wikidata;

use chemical::{convert_smiles, has_undefined_stereo, resolve_exact_mass};
use helpers::{
    binding_value, escape_qs_string, escape_sparql_string, extract_qid_from_uri,
    has_isomeric_smiles, has_stereo_marks, normalize_doi,
};
#[cfg(not(target_arch = "wasm32"))]
use http_client::{BatchConvertResponse, natprod_client};
#[cfg(target_arch = "wasm32")]
use http_client::{js_value_to_json, rdkit_bridge_call};
use reference_metadata::fetch_reference_quickstatements;
use wikidata::normalize_taxon_lookup;

pub mod inputs;
pub mod pipeline;
pub mod quickstatements;

#[cfg(test)]
pub(crate) use chemical::extract_exact_mass_from_json;
pub use enrichment::curate_single_row;
pub use helpers::{extract_formula_from_inchi, normalize_formula_for_wikidata, qs_mass_statement};
