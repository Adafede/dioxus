// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

use crate::query::{
    build_lotus_query_qlever, build_lotus_query_wdqs, build_pubchem_sparql, escape_sparql_literal,
};

fn keys(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn escapes_sparql_literals_in_correct_order() {
    assert_eq!(
        escape_sparql_literal("ABCDEFGHIJKLMN-abcdefghij-1"),
        "ABCDEFGHIJKLMN-abcdefghij-1"
    );
    assert_eq!(escape_sparql_literal(r#"a"b"#), r#"a\"b"#);
    assert_eq!(escape_sparql_literal(r"a\b"), r"a\\b");
    assert_eq!(escape_sparql_literal(r#"a\"b"#), r#"a\\\"b"#);
    assert_eq!(escape_sparql_literal("a\nb\rc\td"), r"a\nb\rc\td");
}

#[test]
fn empty_batches_produce_syntactically_valid_empty_queries() {
    for batch in [keys(&[]), keys(&["", ""])] {
        let pubchem = build_pubchem_sparql(&batch);
        assert!(
            pubchem.contains("WHERE {}") && pubchem.contains("# empty"),
            "{pubchem}"
        );
        assert!(!pubchem.contains("VALUES ?inchikey {"), "{pubchem}");

        let wdqs = build_lotus_query_wdqs(&batch);
        assert!(
            wdqs.contains("WHERE {}") && wdqs.contains("# empty"),
            "{wdqs}"
        );
        assert!(!wdqs.contains("VALUES ?inchikey {"), "{wdqs}");
    }
}

#[test]
fn formats_values_clause_escapes_quotes_and_selects_expected_columns() {
    let wdqs = build_lotus_query_wdqs(&keys(&["K1", "", "K2", r#"a"b"#]));
    assert!(
        wdqs.contains(r#"VALUES ?inchikey { "K1" "K2" "a\"b" }"#),
        "{wdqs}"
    );
    assert_eq!(wdqs.matches("VALUES ?inchikey").count(), 1);
    assert!(wdqs.contains("wdt:P3364") && wdqs.contains("wdt:P6185") && wdqs.contains("wdt:P279"));

    let qlever = build_lotus_query_qlever(&keys(&["K1"]));
    assert!(
        qlever.contains("UNION") && qlever.contains("?prefix_inchikey"),
        "{qlever}"
    );
    assert!(!wdqs.contains("?prefix_inchikey"), "{wdqs}");

    for (name, query) in [("wdqs", wdqs), ("qlever", qlever)] {
        assert!(
            query.contains("OPTIONAL") && query.contains("wdt:P703"),
            "{name}"
        );
        for col in ["?inchikey", "?related_item", "?taxon_name"] {
            assert!(query.contains(col), "{name} missing {col}");
        }
    }

    let pubchem = build_pubchem_sparql(&keys(&["RYYVLZVUVIJVGH-UHFFFAOYSA-N"]));
    assert!(
        pubchem.contains("SELECT DISTINCT ?inchikey ?related_cid"),
        "{pubchem}"
    );
}
