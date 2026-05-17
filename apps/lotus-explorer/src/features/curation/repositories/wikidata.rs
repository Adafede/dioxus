// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::CurationKnowledgeRepository;
use crate::features::curation::domain::{CurationError, WikidataCompound};
use crate::features::curation::services::wikidata;
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Copy)]
pub struct WikidataKnowledgeRepository;

#[async_trait(?Send)]
impl CurationKnowledgeRepository for WikidataKnowledgeRepository {
    async fn fetch_compound_by_inchikey(
        &self,
        inchikey: &str,
    ) -> Result<Option<WikidataCompound>, CurationError> {
        wikidata::fetch_wikidata_compound_by_inchikey(inchikey).await
    }

    async fn resolve_or_create_taxon(
        &self,
        name: &str,
        pre_resolved_qid: Option<&str>,
    ) -> Result<(Option<String>, Vec<String>), CurationError> {
        wikidata::resolve_or_create_taxon(name, pre_resolved_qid).await
    }

    async fn resolve_reference_qid(&self, doi: &str) -> Result<Option<String>, CurationError> {
        wikidata::resolve_reference_qid(doi).await
    }

    async fn compound_has_taxon_with_ref(
        &self,
        compound_qid: &str,
        taxon_qid: &str,
        ref_qid: &str,
    ) -> Result<bool, CurationError> {
        wikidata::compound_has_taxon_with_ref(compound_qid, taxon_qid, ref_qid).await
    }

    async fn compound_has_taxon(
        &self,
        compound_qid: &str,
        taxon_qid: &str,
    ) -> Result<bool, CurationError> {
        wikidata::compound_has_taxon(compound_qid, taxon_qid).await
    }

    async fn resolve_taxon_qids_batch(
        &self,
        names: &[String],
    ) -> Result<HashMap<String, String>, CurationError> {
        wikidata::resolve_taxon_qids_batch(names.iter().map(String::as_str)).await
    }

    async fn resolve_reference_qids_batch(
        &self,
        dois: &[String],
    ) -> Result<HashMap<String, String>, CurationError> {
        wikidata::resolve_reference_qids_batch(dois.iter().map(String::as_str)).await
    }
}
