// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

mod wikidata;

use crate::features::curation::domain::{CurationError, WikidataCompound};
use async_trait::async_trait;
use std::collections::HashMap;

pub use wikidata::WikidataKnowledgeRepository;

/// Stable data-access boundary for curation orchestration and enrichment.
#[async_trait(?Send)]
pub trait CurationKnowledgeRepository: Send + Sync {
    async fn fetch_compound_by_inchikey(
        &self,
        inchikey: &str,
    ) -> Result<Option<WikidataCompound>, CurationError>;

    async fn resolve_or_create_taxon(
        &self,
        name: &str,
        pre_resolved_qid: Option<&str>,
    ) -> Result<(Option<String>, Vec<String>), CurationError>;

    async fn resolve_reference_qid(&self, doi: &str) -> Result<Option<String>, CurationError>;

    async fn compound_has_taxon_with_ref(
        &self,
        compound_qid: &str,
        taxon_qid: &str,
        ref_qid: &str,
    ) -> Result<bool, CurationError>;

    async fn compound_has_taxon(
        &self,
        compound_qid: &str,
        taxon_qid: &str,
    ) -> Result<bool, CurationError>;

    async fn resolve_taxon_qids_batch(
        &self,
        names: &[String],
    ) -> Result<HashMap<String, String>, CurationError>;

    async fn resolve_reference_qids_batch(
        &self,
        dois: &[String],
    ) -> Result<HashMap<String, String>, CurationError>;
}
