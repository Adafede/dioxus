// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Taxon resolution service — maps a free-text name to a Wikidata QID.
//!
//! This module has **no dependency on the Dioxus runtime** and carries **no
//! locale strings**, making every function directly unit-testable.

use crate::features::explore::search_state::SearchMetrics;
use crate::features::explore::types::{
    DomainError, ParseFault, QueryStage, TaxonWarning, ValidationFault,
};
use crate::features::explore::{search_utils::sanitize_taxon_input, taxon_cache};
use crate::models::TaxonMatch;
use crate::perf;
use crate::queries;
use crate::repositories::LotusRepository;
use crate::services::search_telemetry as telemetry;
use crate::sparql;

/// Output of a successful taxon resolution.
#[derive(Debug, Clone)]
pub struct TaxonResolution {
    /// The resolved Wikidata QID (e.g. `"Q12345"`), or `None` if the criteria
    /// contained no taxon, or `Some("*")` for the "all taxa" wildcard.
    pub qid: Option<String>,
    /// Optional structured warning to be formatted at the UI boundary.
    pub warning: Option<TaxonWarning>,
}

/// Resolve a free-text taxon name (or QID, or wildcard) to a Wikidata QID.
///
/// Returns:
/// * `Ok(TaxonResolution { qid: None, .. })` — taxon was blank.
/// * `Ok(TaxonResolution { qid: Some("*"), .. })` — wildcard `"*"` passed through.
/// * `Ok(TaxonResolution { qid: Some(q), .. })` — successfully resolved.
/// * `Err(DomainError::Validation(_))` — taxon not found in Wikidata.
/// * `Err(DomainError::Transport { .. })` — network error during SPARQL lookup.
/// * `Err(DomainError::Parse(_))` — CSV response could not be parsed.
pub async fn resolve<R: LotusRepository>(
    taxon: &str,
    repo: &R,
    metrics: &mut SearchMetrics,
) -> Result<TaxonResolution, DomainError> {
    if taxon.is_empty() {
        return Ok(TaxonResolution {
            qid: None,
            warning: None,
        });
    }
    if taxon == "*" {
        return Ok(TaxonResolution {
            qid: Some("*".to_string()),
            warning: None,
        });
    }
    // Pass a bare Wikidata QID directly — no SPARQL round-trip needed.
    // Accepts both 'Q' and 'q' prefix; the slice `&taxon[1..]` is safe since
    // 'Q'/'q' are single-byte ASCII characters.
    if taxon.starts_with(['Q', 'q']) && taxon[1..].chars().all(|c| c.is_ascii_digit()) {
        return Ok(TaxonResolution {
            qid: Some(taxon.to_uppercase()),
            warning: None,
        });
    }

    let taxon_timer = perf::start_timer("LOTUS:taxon_resolution");
    let sanitized = sanitize_taxon_input(taxon);

    let standardized_warning = if sanitized != taxon {
        Some(TaxonWarning::Standardized {
            original: taxon.to_string(),
            standardized: sanitized.clone(),
        })
    } else {
        None
    };

    // Fast path: cache hit.
    if let Some(cached_qid) = taxon_cache::lookup(&sanitized) {
        let taxon_elapsed = perf::end_timer("LOTUS:taxon_resolution", taxon_timer);
        telemetry::taxon_cache_hit(taxon_elapsed, &sanitized, &cached_qid);
        return Ok(TaxonResolution {
            qid: Some(cached_qid),
            warning: standardized_warning,
        });
    }

    // Slow path: SPARQL query.
    let query = queries::query_taxon_search(&sanitized);
    let csv = repo
        .sparql_bytes(&query)
        .await
        .map_err(DomainError::transport_at(QueryStage::TaxonSearch))?;

    let taxon_elapsed = perf::end_timer("LOTUS:taxon_resolution", taxon_timer);
    metrics.add_network(taxon_elapsed);
    telemetry::taxon_sparql_done(taxon_elapsed);

    let matches = sparql::parse_taxon_csv_bytes(&csv).map_err(|e| {
        DomainError::Parse(ParseFault::TaxonCsv {
            details: e.to_string(),
        })
    })?;

    if matches.is_empty() {
        return Err(DomainError::Validation(ValidationFault::TaxonNotFound {
            input: taxon.to_string(),
        }));
    }

    let lower = sanitized.to_lowercase();
    let exact: Vec<&TaxonMatch> = matches
        .iter()
        .filter(|m| m.name.to_lowercase() == lower)
        .collect();

    let best = exact
        .first()
        .copied()
        .or_else(|| matches.first())
        .ok_or(DomainError::Parse(ParseFault::TaxonPick {
            details: "no candidates after parse".to_string(),
        }))?;

    let warning = if exact.len() > 1 || (exact.is_empty() && matches.len() > 1) {
        let candidates = matches
            .iter()
            .take(4)
            .map(|m| format!("{} ({})", m.name, m.qid))
            .collect::<Vec<_>>();
        Some(TaxonWarning::Ambiguous {
            chosen_name: best.name.clone(),
            chosen_qid: best.qid.clone(),
            candidates,
        })
    } else {
        standardized_warning
    };

    taxon_cache::store(&sanitized, &best.qid);
    Ok(TaxonResolution {
        qid: Some(best.qid.clone()),
        warning,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::SearchResponse;
    use crate::models::SearchCriteria;
    use crate::repositories::{LotusRepository, RepositoryError};

    /// Stub that always returns a fixed SPARQL CSV response; API not configured.
    #[derive(Clone)]
    struct StubRepo {
        response: Result<Vec<u8>, RepositoryError>,
    }

    impl StubRepo {
        fn ok(csv: &str) -> Self {
            Self {
                response: Ok(csv.as_bytes().to_vec()),
            }
        }
        fn err_network(msg: &str) -> Self {
            Self {
                response: Err(RepositoryError::network(msg.to_string())),
            }
        }
    }

    impl LotusRepository for StubRepo {
        async fn api_search(
            &self,
            _: &SearchCriteria,
            _: usize,
            _: bool,
        ) -> Option<Result<SearchResponse, RepositoryError>> {
            None
        }

        async fn sparql_bytes(&self, _: &str) -> Result<Vec<u8>, RepositoryError> {
            self.response.clone()
        }
    }

    #[test]
    fn empty_taxon_returns_none_qid() {
        let result = futures::executor::block_on(resolve(
            "",
            &StubRepo::ok(""),
            &mut SearchMetrics::default(),
        ));
        let r = result.unwrap();
        assert!(r.qid.is_none());
        assert!(r.warning.is_none());
    }

    #[test]
    fn star_taxon_returns_star() {
        let r = futures::executor::block_on(resolve(
            "*",
            &StubRepo::ok(""),
            &mut SearchMetrics::default(),
        ))
        .unwrap();
        assert_eq!(r.qid.as_deref(), Some("*"));
    }

    #[test]
    fn q_prefix_taxon_passes_through_uppercase() {
        let r = futures::executor::block_on(resolve(
            "q12345",
            &StubRepo::ok(""),
            &mut SearchMetrics::default(),
        ))
        .unwrap();
        assert_eq!(r.qid.as_deref(), Some("Q12345"));
    }

    #[test]
    fn network_error_becomes_transport_domain_error() {
        // Clear cache to ensure SPARQL path is taken.
        let result = futures::executor::block_on(resolve(
            "Completely Unknown Taxon XYZ Unique",
            &StubRepo::err_network("timeout"),
            &mut SearchMetrics::default(),
        ));
        assert!(
            matches!(
                result,
                Err(DomainError::Transport {
                    stage: QueryStage::TaxonSearch,
                    ..
                })
            ),
            "expected Transport error, got: {result:?}"
        );
    }

    #[test]
    fn empty_sparql_result_returns_taxon_not_found() {
        // CSV with only the header row → zero matches.
        let csv = "taxon,taxonLabel\n";
        let result = futures::executor::block_on(resolve(
            "Nonexistent Plant ABC",
            &StubRepo::ok(csv),
            &mut SearchMetrics::default(),
        ));
        assert!(
            matches!(
                result,
                Err(DomainError::Validation(
                    ValidationFault::TaxonNotFound { .. }
                ))
            ),
            "expected TaxonNotFound, got: {result:?}"
        );
    }
}
