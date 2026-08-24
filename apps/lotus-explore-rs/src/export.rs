// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Export helpers split by concern:
//! - `filters`: criteria -> structured JSON
//! - `metadata`: schema.org dataset metadata generation
//! - `filename`: deterministic download filenames and timestamp helpers
//! - `urls`: QLever and WDQS export URL generation

mod filename;
mod filters;
mod metadata;
mod url_builder;

pub use filename::generate_filename;
pub use metadata::{MetadataInputs, SparqlEndpoint, build_metadata_json};
pub use url_builder::{qlever_export_url, wdqs_export_url, wdqs_scholarly_export_url};
