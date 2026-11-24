// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

use crate::qlever::enrich_sources;
use crate::sparql::{QLEVER_WIKIDATA, WDQS_WIKIDATA};
use futures_executor::block_on;
use lotus_search::testing::Scripted;

const ASK_OK: &str = r#"{"boolean":true}"#;
const LOTUS_BINDINGS: &str = r#"{
  "results": {
    "bindings": [
      {
        "inchikey": {"value": "RYYVLZVUVIJVGH-UHFFFAOYSA-N"},
        "related_item": {"value": "http://www.wikidata.org/entity/Q60235"},
        "taxon_name": {"value": "Coffea arabica"}
      },
      {
        "inchikey": {"value": "RYYVLZVUVIJVGH-UHFFFAOYSA-N"},
        "related_item": {"value": "http://www.wikidata.org/entity/Q999999"}
      }
    ]
  }
}"#;
const PUBCHEM_BINDINGS: &str = r#"{
  "results": {
    "bindings": [
      {
        "inchikey": {"value": "RYYVLZVUVIJVGH-UHFFFAOYSA-N"},
        "related_cid": {"value": "2519"}
      }
    ]
  }
}"#;

#[test]
fn enrich_sources_parses_wdqs_and_pubchem_hits() {
    let http = Scripted::new(vec![
        (200, ASK_OK),
        (200, ASK_OK),
        (200, LOTUS_BINDINGS),
        (200, PUBCHEM_BINDINGS),
    ]);
    let mut statuses = Vec::new();
    let outcome = block_on(enrich_sources(
        &http,
        &["RYYVLZVUVIJVGH-UHFFFAOYSA-N".to_string()],
        |msg| statuses.push(msg.to_string()),
    ));

    assert!(
        outcome.warnings.is_empty(),
        "warnings: {:?}",
        outcome.warnings
    );
    assert_eq!(outcome.endpoints.len(), 3);
    assert!(outcome.endpoints.iter().all(|ep| ep.reachable));
    assert_eq!(
        outcome.endpoints.first().map(|ep| ep.endpoint.as_str()),
        Some(WDQS_WIKIDATA)
    );

    let lotus = outcome.enrichment.lotus.get("RYYVLZVUVIJVGH");
    assert!(
        lotus.is_some(),
        "LOTUS connectivity key should be populated"
    );
    if let Some(summary) = lotus {
        assert!(summary.taxa.contains("Coffea arabica"));
        assert!(summary.compounds.contains("Q60235"));
        assert!(summary.compounds.contains("Q999999"));
        assert!(summary.compounds_with_taxa.contains("Q60235"));
        assert!(!summary.compounds_with_taxa.contains("Q999999"));
    }

    let pubchem = outcome.enrichment.pubchem.get("RYYVLZVUVIJVGH");
    assert!(pubchem.is_some_and(|s| s.cids.contains("2519")));
    assert_ne!(statuses, [] as [std::string::String; 0]);
}

#[test]
fn enrich_sources_falls_back_to_qlever_when_wdqs_rejects_and_reports_unreachable_pubchem() {
    let http = Scripted::new(vec![
        (200, ASK_OK),
        (404, "missing pubchem"),
        (400, "WDQS rejected query"),
        (200, LOTUS_BINDINGS),
    ]);
    let outcome = block_on(enrich_sources(
        &http,
        &["RYYVLZVUVIJVGH-UHFFFAOYSA-N".to_string()],
        |_| {},
    ));

    assert_eq!(
        outcome.endpoints.first().map(|ep| ep.endpoint.as_str()),
        Some(QLEVER_WIKIDATA)
    );
    assert!(outcome.endpoints.first().is_some_and(|ep| ep.reachable));
    assert!(
        outcome
            .warnings
            .iter()
            .any(|w| w.contains("switched to Qlever fallback")),
        "warnings: {:?}",
        outcome.warnings
    );
    assert!(
        outcome
            .warnings
            .iter()
            .any(|w| w.contains("PubChem endpoint unavailable")),
        "warnings: {:?}",
        outcome.warnings
    );
    assert!(outcome.enrichment.pubchem.is_empty());
}

#[test]
fn enrich_sources_records_warning_when_both_lotus_endpoints_fail() {
    let http = Scripted::new(vec![
        (200, ASK_OK),
        (200, ASK_OK),
        (400, "wdqs bad"),
        (400, "qlever bad"),
        (200, r#"{"results":{"bindings":[]}}"#),
    ]);
    let outcome = block_on(enrich_sources(
        &http,
        &["RYYVLZVUVIJVGH-UHFFFAOYSA-N".to_string()],
        |_| {},
    ));

    assert!(outcome.endpoints.first().is_some_and(|ep| !ep.reachable));
    assert!(
        outcome
            .warnings
            .iter()
            .any(|w| w.contains("LOTUS qlever also failed")),
        "warnings: {:?}",
        outcome.warnings
    );
}
