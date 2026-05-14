// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

mod constants;
mod internal;
mod quickstatements;
mod types;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use constants::NATPROD_API_BASE;
pub(crate) use constants::{
    CURATION_SPARQL_PREFIXES, WD_CHEMICAL_COMPOUND_QID, WD_OCCURS_IN_TAXON_PROP,
    WD_STEREOISOMER_GROUP_QID, WD_TAXON_QID, WD_TYPE_CHEMICAL_ENTITY_QID,
};
pub(crate) use internal::{DependencyResolution, MassResolution, WikidataCompound};
pub use quickstatements::build_quickstatements_bundle;
pub use types::{
    CurationError, CurationInputRow, CurationResultRow, CurationStatus, QuickStatementsBundle,
};
