// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

use super::{columns, count_str};
use crate::JsonCountError;

#[test]
fn top_level_scalars_and_containers_match_expected_counts() {
    for (input, expected) in [
        ("", 0),
        ("{}", 0),
        ("[]", 0),
        ("42", 1),
        ("true", 1),
        ("false", 1),
        ("null", 0),
        (r#""""#, 0),
        (r#""hello""#, 1),
        (r#"{"a":1,"b":null}"#, 1),
        (r#"{"a":null,"b":null}"#, 0),
        (r#"{"a":"hello"}"#, 1),
        (r#"{"a":""}"#, 0),
        (r#"{"a":true,"b":42,"c":false}"#, 3),
        (r#"{"a":1,"e":{"f":42}}"#, 2),
        (r#"{"items":[1,2,3]}"#, 3),
        (r#"{"a":{"b":{"c":[1,null]}},"d":true}"#, 2),
        (r#"{"a":"hello\nworld"}"#, 1),
        (r#"{ "a" : 1 , "b" : null , "c" : true }"#, 2),
        (r#"{"a":1}  "#, 1),
        (r#"{"a":[[1]]}"#, 1),
    ] {
        assert_eq!(count_str(input), Ok(expected), "count failed for {input}");
    }
    assert!(
        matches!(
            count_str(r#"{"a":1"#),
            Err(JsonCountError::UnexpectedEof(_))
        ),
        "truncated container returns UnexpectedEof"
    );
}

#[test]
fn column_object_and_empty_string_rules() {
    assert_eq!(
        columns(r#"{"a":1,"b":2,"c":3}"#, 64),
        Ok(vec![
            ("a".to_string(), 1),
            ("b".to_string(), 1),
            ("c".to_string(), 1)
        ]),
        "one entry per key"
    );
    assert_eq!(
        columns(r#"{"x":[1,null,2,{"y":3},null]}"#, 64),
        Ok(vec![("x".to_string(), 3)]),
        "nested keys do not count; only non-null leaves count"
    );
    assert_eq!(
        columns("   \t\n ", 4),
        Ok(Vec::new()),
        "whitespace-only input has no columns"
    );
    assert_eq!(
        columns(r#"{"":1,"b":""}"#, 4),
        Ok(vec![(String::new(), 1), ("b".to_string(), 0)]),
        "top-level empty key allowed"
    );
    assert_eq!(
        columns(r#"{"x":[{"":1}]}"#, 64),
        Ok(vec![("x".to_string(), 1)]),
        "nested empty key counts 0, value 1"
    );
    assert_eq!(
        columns(r#"{"x":["\\"]}"#, 64),
        Ok(vec![("x".to_string(), 1)]),
        "escaped backslash string is non-empty"
    );
    assert_eq!(columns("{}", 64), Ok(Vec::new()), "empty object");
    assert_eq!(columns("[]", 64), Ok(Vec::new()), "empty array");
}

#[test]
fn row_arrays_and_duplicate_keys_count_correctly() {
    let doc = r#"[{"id":"0","qc":null,"peaks":[[91.0,0.1],[125.0,0.3]]},{},{"id":"1","qc":"ok","peaks":[]},{"id":"2","qc":"","peaks":[[100.0,1.0]]}]"#;
    let whole = Ok(vec![
        ("id".to_string(), 3),
        ("qc".to_string(), 1),
        ("peaks".to_string(), 2),
    ]);
    for chunk in [1, 2, 3, 5, 7, 13, 64, doc.len()] {
        assert_eq!(
            columns(doc, chunk),
            whole,
            "chunk size {chunk} changed row-array result"
        );
    }

    for chunk in [1, 4, 64] {
        assert_eq!(
            columns(r#"[{"a":1,"a":2},{"a":3}]"#, chunk),
            Ok(vec![("a".to_string(), 2)]),
            "at most once per row"
        );
        assert_eq!(
            columns(r#"[{"a":null,"a":2}]"#, chunk),
            Ok(vec![("a".to_string(), 1)]),
            "null then non-null in same row"
        );
        assert_eq!(
            columns(r#"{"a":1,"a":2}"#, chunk),
            Ok(vec![("a".to_string(), 2)]),
            "duplicate top-level key sums"
        );
    }
}

#[test]
fn escaped_quotes_and_backslashes_in_keys_and_values() {
    assert_eq!(
        columns(r#"{"k":"a\"b","z":"c\\","y":null}"#, 4),
        Ok(vec![
            ("k".to_string(), 1),
            ("z".to_string(), 1),
            ("y".to_string(), 0)
        ]),
        "escaped quotes and backslashes do not corrupt following columns"
    );
    assert_eq!(
        columns(r#"{"a\"b":1,"c\u0041d":2}"#, 4),
        Ok(vec![("a\"b".to_string(), 1), ("cAd".to_string(), 1)]),
        "escaped key names preserved"
    );
}

#[test]
fn malformed_json_and_limits_are_rejected() {
    assert_eq!(
        columns("[1,2,3]", 64),
        Err(JsonCountError::ExpectedTopLevelObject),
        "non-object array items rejected"
    );
    assert_eq!(
        columns(r#"{"k" 1}"#, 64),
        Err(JsonCountError::ExpectedColon),
        "missing colon rejected"
    );
    for input in [
        r#"{"a":1,}"#,
        r#"[{"a":1},]"#,
        r#"{"a":1"#,
        r#"[{"a":1}"#,
        r#"{"a":1 "b":2}"#,
        r#"[{"a":1}{"b":2}]"#,
        r#"{"a":1]}"#,
        "{}{}",
        "[]1",
        r#"{"a":[}]}"#,
        r#"{"a":{]}}"#,
        r#"{"a":nulX}"#,
        r#"{"a":[nul]}"#,
        r#"{"a":tr}"#,
        r#"{"a":-}"#,
        r#"{"a":+1}"#,
        r#"{"a":1.}"#,
        r#"{"a":1./}"#,
        r#"{"a":1e+}"#,
        r#"{"a":01}"#,
        r#"{"k":"a\"#,
        r#"{"k":"abc"#,
        r#"{"k":1"#,
        r#"{"a":[1,2,"b":3]}"#,
        r#"{"a":{"b"}}"#,
        r#"{"a":{"b":1,}}"#,
        r#"{"a":[1,]}"#,
        r#"{"a":[1 2]}"#,
        r#"{"a":{"x" "y":1}}"#,
        r#"{"a":{"x":1 "y":2}}"#,
        "{\"k\":\"bad\nchar\"}",
        r#"{"k":"bad\qescape"}"#,
        r#"{"k":"\u+041"}"#,
        r#"{"k":{"nested":"\u00zz"}}"#,
    ] {
        assert!(
            columns(input, 4).is_err(),
            "expected syntax error for {input}"
        );
    }

    let bad_utf8 = b"{\"k\":\"\xff\"}";
    assert_eq!(
        futures_executor::block_on(crate::scan_source_with_progress(
            stream_io::SliceSource::new(bad_utf8),
            bad_utf8.len() as u64,
            |_, _| {}
        )),
        Err(JsonCountError::InvalidUtf8),
        "invalid UTF-8 inside string value rejected"
    );
    assert_eq!(
        columns(r#"{"a":{"b":{"c":[1,{"k":"v"},null,false]}}}"#, 2),
        Ok(vec![("a".to_string(), 3)]),
        "deep nested objects/arrays count only non-null leaves (1, \"v\", and false)"
    );

    let max_scalar = "1".repeat(65_536);
    assert_eq!(
        columns(&format!(r#"{{"n":{max_scalar}}}"#), 65_536),
        Ok(vec![("n".to_string(), 1)]),
        "64 KiB scalar allowed"
    );
    let over_scalar = "1".repeat(65_537);
    assert_eq!(
        columns(&format!(r#"{{"n":{over_scalar}}}"#), 65_536),
        Err(JsonCountError::ScalarTooLong),
        "scalar over 64 KiB rejected"
    );

    let max_key = "k".repeat(1_048_576);
    assert_eq!(
        columns(&format!(r#"{{"{max_key}":1}}"#), 65_536),
        Ok(vec![(max_key, 1)]),
        "1 MiB key allowed"
    );
    let over_key = "k".repeat(1_048_577);
    assert_eq!(
        columns(&format!(r#"{{"{over_key}":1}}"#), 65_536),
        Err(JsonCountError::KeyTooLong),
        "key over 1 MiB rejected"
    );
}
