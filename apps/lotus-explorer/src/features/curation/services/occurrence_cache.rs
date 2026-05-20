// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::curation::domain::CurationError;
use crate::features::curation::repositories::CurationKnowledgeRepository;
use std::collections::HashMap;
use std::sync::Mutex;

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

pub(crate) async fn compound_has_taxon_cached(
    repository: &dyn CurationKnowledgeRepository,
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

    let value = repository
        .compound_has_taxon(compound_qid, taxon_qid)
        .await?;
    write_cached_ask(cache, key, value);
    Ok(value)
}

pub(crate) async fn compound_has_taxon_with_ref_cached(
    repository: &dyn CurationKnowledgeRepository,
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

    let value = repository
        .compound_has_taxon_with_ref(compound_qid, taxon_qid, ref_qid)
        .await?;
    write_cached_ask(cache, key, value);
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::curation::domain::WikidataCompound;
    use async_trait::async_trait;
    use futures::executor::block_on;

    #[derive(Default)]
    struct MockRepo {
        ask_calls: Mutex<usize>,
    }

    #[async_trait(?Send)]
    impl CurationKnowledgeRepository for MockRepo {
        async fn fetch_compound_by_inchikey(
            &self,
            _inchikey: &str,
        ) -> Result<Option<WikidataCompound>, CurationError> {
            Ok(None)
        }

        async fn resolve_or_create_taxon(
            &self,
            _name: &str,
            _pre_resolved_qid: Option<&str>,
        ) -> Result<(Option<String>, Vec<String>), CurationError> {
            Ok((None, Vec::new()))
        }

        async fn resolve_reference_qid(&self, _doi: &str) -> Result<Option<String>, CurationError> {
            Ok(None)
        }

        async fn compound_has_taxon_with_ref(
            &self,
            _compound_qid: &str,
            _taxon_qid: &str,
            _ref_qid: &str,
        ) -> Result<bool, CurationError> {
            Ok(false)
        }

        async fn compound_has_taxon(
            &self,
            _compound_qid: &str,
            _taxon_qid: &str,
        ) -> Result<bool, CurationError> {
            if let Ok(mut calls) = self.ask_calls.lock() {
                *calls += 1;
            }
            Ok(false)
        }

        async fn resolve_taxon_qids_batch(
            &self,
            _names: &[String],
        ) -> Result<HashMap<String, String>, CurationError> {
            Ok(HashMap::new())
        }

        async fn resolve_reference_qids_batch(
            &self,
            _dois: &[String],
        ) -> Result<HashMap<String, String>, CurationError> {
            Ok(HashMap::new())
        }
    }

    #[test]
    fn ask_cache_reuses_result_for_same_compound_taxon_pair() {
        let repo = MockRepo::default();
        let cache = Mutex::new(OccurrenceAskCache::default());

        let first = block_on(compound_has_taxon_cached(&repo, &cache, "Q1", "Q2"))
            .expect("first ask result");
        let second = block_on(compound_has_taxon_cached(&repo, &cache, "Q1", "Q2"))
            .expect("second ask result");

        assert!(!first);
        assert!(!second);
        assert_eq!(*repo.ask_calls.lock().expect("counter lock"), 1);
    }
}
