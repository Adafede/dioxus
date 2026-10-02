// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! The SPARQL this app sends, built from a list of InChIKeys.
//!
//! These three functions build strings and nothing else — no browser API, no
//! HTTP, no Dioxus — and they are deliberately ungated, because `qlever`, which
//! owns the transport and is `#[cfg(target_arch = "wasm32")]`, is the only thing
//! that combines them. A host build compiles none of that, so a query malformed
//! in a way only the endpoint notices would otherwise be the one failure with no
//! test.

/// Build Wikidata LOTUS query for WDQS (standard nested SELECT approach).
pub(crate) fn build_lotus_query_wdqs(inchikeys: &[String]) -> String {
    let values = inchikeys
        .iter()
        .filter(|s| !s.is_empty())
        .map(|v| format!("\"{}\"", escape_sparql_literal(v)))
        .collect::<Vec<_>>()
        .join(" ");

    if values.is_empty() {
        return "SELECT DISTINCT ?inchikey ?related_item ?taxon_name WHERE {} # empty".to_string();
    }

    // Nested SELECT structure for cleaner query
    format!(
        r#"PREFIX wdt: <http://www.wikidata.org/prop/direct/>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
SELECT DISTINCT ?inchikey ?related_item ?taxon_name WHERE {{
  {{
    SELECT DISTINCT ?inchikey ?connectivity ?related_item WHERE {{
      VALUES ?inchikey {{ {values} }}
      BIND(SUBSTR(?inchikey, 1 , 14 ) AS ?connectivity)
      ?item wdt:P235 ?inchikey;
        (wdt:P3364|wdt:P6185|(wdt:P279*)|^(wdt:P279+)) ?related_item.
      OPTIONAL {{ ?related_item wdt:P235 ?related_inchikey. }}
      OPTIONAL {{
        ?item wdt:P6185 ?related_item.
        BIND("true"^^xsd:boolean AS ?is_tautomer)
      }}
      FILTER(((?item = ?related_item) || (BOUND(?is_tautomer))) || (STRSTARTS(?related_inchikey, ?connectivity)))
    }}
  }}
  OPTIONAL {{ ?related_item (wdt:P703/wdt:P225) ?taxon_name. }}
}}"#
    )
}

/// Build Wikidata LOTUS query for qlever (optimized union structure).
pub(crate) fn build_lotus_query_qlever(inchikeys: &[String]) -> String {
    let values = inchikeys
        .iter()
        .filter(|s| !s.is_empty())
        .map(|v| format!("\"{}\"", escape_sparql_literal(v)))
        .collect::<Vec<_>>()
        .join(" ");

    if values.is_empty() {
        return "SELECT DISTINCT ?inchikey ?related_item ?taxon_name WHERE {} # empty".to_string();
    }

    // Union-based query for qlever optimization
    format!(
        r#"PREFIX wdt: <http://www.wikidata.org/prop/direct/>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

SELECT DISTINCT ?inchikey ?related_item ?taxon_name WHERE {{
  {{
    SELECT DISTINCT ?inchikey ?connectivity ?related_item WHERE {{
      VALUES ?inchikey {{ {values} }}
      BIND(SUBSTR(?inchikey, 1, 14) AS ?connectivity)
      ?item wdt:P235 ?inchikey .

      {{
        ?item (wdt:P3364|wdt:P6185|(wdt:P279*)|^(wdt:P279+)) ?related_item .
        OPTIONAL {{ ?related_item wdt:P235 ?related_inchikey . }}
        OPTIONAL {{
          ?item wdt:P6185 ?related_item .
          BIND("true"^^xsd:boolean AS ?is_tautomer)
        }}
        FILTER(((?item = ?related_item) || (BOUND(?is_tautomer))) || (STRSTARTS(?related_inchikey, ?connectivity)))
      }}
      UNION
      {{
        # Native QLever compressed dictionary prefix lookup
        ?related_item wdt:P235 ?prefix_inchikey .
        FILTER(STRSTARTS(?prefix_inchikey, ?connectivity))
      }}
    }}
  }}
  OPTIONAL {{ ?related_item (wdt:P703/wdt:P225) ?taxon_name . }}
}}"#
    )
}

pub(crate) fn escape_sparql_literal(value: &str) -> String {
    value.replace('\\', r"\\").replace('"', r#"\""#)
}

#[cfg(test)]
mod tests;
