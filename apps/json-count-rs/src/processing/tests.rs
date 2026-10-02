// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

// The `tests` tests, extracted from `processing.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `processing` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;

#[test]
fn empty_string_counts_zero() {
    assert_eq!(count_non_null_leaves(""), 0);
}

#[test]
fn empty_object_counts_zero() {
    assert_eq!(count_non_null_leaves("{}"), 0);
}

#[test]
fn empty_array_counts_zero() {
    assert_eq!(count_non_null_leaves("[]"), 0);
}

#[test]
fn flat_object_counts_values_and_keys() {
    // Keys ("a", "b") count as 1 each; non-null value 1 counts as 1;
    // null value counts as 0. Total: 3.
    assert_eq!(count_non_null_leaves(r#"{"a":1,"b":null}"#), 3);
}

#[test]
fn all_null_object_counts_zero() {
    // Keys are counted but null values are not.
    assert_eq!(count_non_null_leaves(r#"{"a":null,"b":null}"#), 2);
}

#[test]
fn string_values_counted() {
    // "a" (key), "hello" (value) → 2.
    assert_eq!(count_non_null_leaves(r#"{"a":"hello"}"#), 2);
}

#[test]
fn empty_string_value_not_counted() {
    // "a" (key) → 1; "" (empty value) → 0.
    assert_eq!(count_non_null_leaves(r#"{"a":""}"#), 1);
}

#[test]
fn boolean_and_numbers_counted() {
    // "a"(1), true(1), "b"(1), 42(1), "c"(1), false(1) → 6
    assert_eq!(count_non_null_leaves(r#"{"a":true,"b":42,"c":false}"#), 6);
}

#[test]
fn null_counted_as_zero() {
    // "a"(1), null(0) → 1
    assert_eq!(count_non_null_leaves(r#"{"a":null}"#), 1);
}

#[test]
fn nested_object_counts_recursively() {
    // "a"(1), 1(1), "e"(1), "f"(1), 42(1) → 5
    assert_eq!(count_non_null_leaves(r#"{"a":1,"e":{"f":42}}"#), 5);
}

#[test]
fn array_values_counted() {
    // "items"(1), 1(1), 2(1), 3(1) → 4
    assert_eq!(count_non_null_leaves(r#"{"items":[1,2,3]}"#), 4);
}

#[test]
fn nested_arrays_counted() {
    // "a"(1), "b"(1), 1(1), 2(1), 3(1) → 5
    assert_eq!(count_non_null_leaves(r#"{"a":[1,2,"b":3]}"#), 5);
}

#[test]
fn deeply_nested_structure() {
    // "a"(1), "b"(1), "c"(1), 1(1), null(0), "d"(1), true(1) → 6
    let json = r#"{"a":{"b":{"c":[1,null]}},"d":true}"#;
    assert_eq!(count_non_null_leaves(json), 6);
}

#[test]
fn escaped_strings_are_non_empty() {
    // "a"(1), "hello\nworld"(1) → 2
    assert_eq!(count_non_null_leaves(r#"{"a":"hello\nworld"}"#), 2);
}

#[test]
fn whitespace_between_tokens() {
    let json = r#"{ "a" : 1 , "b" : null , "c" : true }"#;
    // "a"(1), 1(1), "b"(1), null(0), "c"(1), true(1) → 5
    assert_eq!(count_non_null_leaves(json), 5);
}

#[test]
fn top_level_scalar_counts() {
    assert_eq!(count_non_null_leaves("42"), 1);
    assert_eq!(count_non_null_leaves("true"), 1);
    assert_eq!(count_non_null_leaves("false"), 1);
    assert_eq!(count_non_null_leaves("null"), 0);
}

#[test]
fn top_level_empty_string_counts_zero() {
    assert_eq!(count_non_null_leaves(r#"""#), 0);
}

#[test]
fn top_level_string_counts_one() {
    assert_eq!(count_non_null_leaves(r#""hello""#), 1);
}

#[test]
fn trailing_whitespace_ignored() {
    assert_eq!(count_non_null_leaves(r#"{"a":1}  "#), 2);
}

#[test]
fn truncated_container_returns_partial() {
    // No closing brace — scanner reaches EOF and returns what it has.
    // "a"(1), 1(1) → 2
    assert_eq!(count_non_null_leaves(r#"{"a":1"#), 2);
}

#[test]
fn deeply_nested_array() {
    // "a"(1) + [[1]] → "a"(1), inner value 1(1) = 2 (arrays themselves don't count)
    assert_eq!(count_non_null_leaves(r#"{"a":[[1]]}"#), 2);
}
