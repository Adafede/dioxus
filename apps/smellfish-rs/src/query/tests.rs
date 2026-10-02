// The `tests` tests, extracted from `query.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `query` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;

fn keys(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

// ── escape_sparql_literal ───────────────────────────────────────────────

#[test]
fn an_ordinary_value_is_untouched() {
    // An InChIKey is alphanumerics and dashes, so this is every real input.
    assert_eq!(
        escape_sparql_literal("ABCDEFGHIJKLMN-abcdefghij-1"),
        "ABCDEFGHIJKLMN-abcdefghij-1",
        "nothing to escape"
    );
}

#[test]
fn a_quote_is_escaped() {
    // An unescaped quote would end the SPARQL string literal early and the
    // rest of the value would be read as query syntax.
    assert_eq!(escape_sparql_literal(r#"a"b"#), r#"a\"b"#, "quote escaped");
}

#[test]
fn a_backslash_is_escaped() {
    assert_eq!(escape_sparql_literal(r"a\b"), r"a\\b", "backslash escaped");
}

#[test]
fn the_backslash_is_escaped_before_the_quote() {
    // Order matters. Escaping the quote first would turn `\"` (an escaped
    // quote) into `\\"`, which is an escaped backslash followed by a
    // *string-ending* quote — the opposite of what was meant.
    assert_eq!(
        escape_sparql_literal(r#"a\"b"#),
        r#"a\\\"b"#,
        "both escaped, backslash first"
    );
}

#[test]
fn a_newline_is_not_escaped() {
    // A gap, stated rather than hidden: a literal newline inside a SPARQL
    // string is legal, so this is harmless, but it means the escaper is not
    // a general string-literal encoder. An InChIKey cannot contain one.
    assert_eq!(
        escape_sparql_literal("a\nb"),
        "a\nb",
        "a newline passes through: legal in a SPARQL literal"
    );
}

// ── the VALUES clause both builders share ──────────────────────────────

#[test]
fn an_empty_key_list_produces_a_query_that_matches_nothing() {
    // Not an empty string and not a bare `VALUES {}`: a syntactically valid
    // query with an empty pattern. Sending `VALUES { }` would be a syntax
    // error, and sending the pattern with no VALUES would ask the endpoint
    // for every compound on Wikidata.
    let q = build_lotus_query_wdqs(&[]);
    assert!(q.contains("WHERE {}"), "an empty pattern: {q}");
    assert!(q.contains("# empty"), "and it says so in a comment: {q}");
    assert!(
        !q.contains("VALUES ?inchikey {"),
        "there is no VALUES clause to be empty: {q}"
    );
}

#[test]
fn a_list_of_only_empty_keys_is_treated_as_empty() {
    // The `filter(|s| !s.is_empty())` runs before the emptiness check, so
    // keys that are present but blank leave nothing to ask about.
    let q = build_lotus_query_wdqs(&keys(&["", ""]));
    assert!(q.contains("WHERE {}"), "blank keys are no keys: {q}");
}

#[test]
fn one_key_becomes_one_value() {
    let q = build_lotus_query_wdqs(&keys(&["AAAAAAAAAAAAAA-BBBBBBBBBB-1"]));
    assert!(
        q.contains(r#"VALUES ?inchikey { "AAAAAAAAAAAAAA-BBBBBBBBBB-1" }"#),
        "the key is quoted inside the VALUES clause: {q}"
    );
}

#[test]
fn several_keys_are_space_separated_in_one_clause() {
    // One clause with N values, not N clauses: a second `VALUES` would
    // override the first rather than add to it.
    let q = build_lotus_query_wdqs(&keys(&["K1", "K2", "K3"]));
    assert!(
        q.contains(r#"VALUES ?inchikey { "K1" "K2" "K3" }"#),
        "all three in one clause: {q}"
    );
    assert_eq!(
        q.matches("VALUES ?inchikey").count(),
        1,
        "and only one such clause: {q}"
    );
}

#[test]
fn blank_keys_are_dropped_from_the_middle_of_a_list() {
    let q = build_lotus_query_wdqs(&keys(&["K1", "", "K2"]));
    assert!(
        q.contains(r#"VALUES ?inchikey { "K1" "K2" }"#),
        "the blank leaves no gap: {q}"
    );
}

#[test]
fn a_key_containing_a_quote_is_escaped_in_both_builders() {
    for (name, q) in [
        ("wdqs", build_lotus_query_wdqs(&keys(&[r#"a"b"#]))),
        ("qlever", build_lotus_query_qlever(&keys(&[r#"a"b"#]))),
    ] {
        assert!(
            q.contains(r#""a\"b""#),
            "{name} escapes the quote inside the literal: {q}"
        );
    }
}

// ── the two builders differ where they are meant to ─────────────────────

#[test]
fn the_wdqs_query_asks_for_the_compound_itself_and_its_relatives() {
    let q = build_lotus_query_wdqs(&keys(&["K1"]));
    // The stereoisomer/tautomer/parent alternation, which is the whole
    // reason for the nested SELECT.
    assert!(q.contains("wdt:P3364"), "stereoisomers: {q}");
    assert!(q.contains("wdt:P6185"), "tautomers: {q}");
    assert!(q.contains("wdt:P279"), "parents: {q}");
    assert!(
        q.contains("SUBSTR(?inchikey, 1 , 14 )"),
        "and the 14-character skeleton key: {q}"
    );
}

#[test]
fn the_qlever_query_also_looks_the_key_up_as_a_prefix() {
    // QLever has a compressed dictionary, so a prefix match is far cheaper
    // than the property path. This union is the reason there are two
    // builders at all.
    let q = build_lotus_query_qlever(&keys(&["K1"]));
    assert!(q.contains("UNION"), "the two strategies are unioned: {q}");
    assert!(
        q.contains("STRSTARTS(?prefix_inchikey, ?connectivity)"),
        "one of them is a prefix lookup: {q}"
    );
}

#[test]
fn only_the_qlever_query_has_the_prefix_branch() {
    // The reason there are two builders. QLever's compressed dictionary
    // makes a prefix lookup cheap, so its query unions the two strategies;
    // WDQS has no such dictionary and the prefix branch would only be a
    // second, much slower way to find the same answers.
    let qlev = build_lotus_query_qlever(&keys(&["K1"]));
    let wdqs = build_lotus_query_wdqs(&keys(&["K1"]));
    assert!(
        qlev.contains("?prefix_inchikey"),
        "qlever has the prefix branch: {qlev}"
    );
    assert!(!wdqs.contains("?prefix_inchikey"), "wdqs does not: {wdqs}");
}

#[test]
fn both_queries_ask_for_the_three_columns_the_reader_expects() {
    // `inchikey`, `related_item` and `taxon_name` are what
    // `fetch_lotus_hits_by_inchikey` then reads out of each binding, so a
    // column renamed here is a column the reader cannot find.
    for (name, q) in [
        ("wdqs", build_lotus_query_wdqs(&keys(&["K1"]))),
        ("qlever", build_lotus_query_qlever(&keys(&["K1"]))),
    ] {
        for column in ["?inchikey", "?related_item", "?taxon_name"] {
            assert!(q.contains(column), "{name} selects {column}: {q}");
        }
    }
}

#[test]
fn both_queries_derive_the_taxon_from_the_related_item() {
    // The taxon is an OPTIONAL, because a compound with no taxon is still a
    // hit. Making it required would drop every unannotated compound.
    for (name, q) in [
        ("wdqs", build_lotus_query_wdqs(&keys(&["K1"]))),
        ("qlever", build_lotus_query_qlever(&keys(&["K1"]))),
    ] {
        assert!(q.contains("OPTIONAL"), "{name} has optional parts: {q}");
        assert!(
            q.contains("wdt:P703"),
            "{name} asks for the taxon source: {q}"
        );
    }
}
