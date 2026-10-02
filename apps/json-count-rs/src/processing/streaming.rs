// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

// The `streaming` tests, extracted from `processing.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `processing` module, so
// `use super::*` below reaches exactly what it did before the move.
use futures_executor::block_on;
use upload::{BlobCursor, SliceSource};

use super::scan_columns;

/// A cursor over `bytes` with a deliberately tiny chunk size.
fn cursor(bytes: &[u8], chunk: usize) -> BlobCursor<fn(u64, u64), SliceSource<'_>> {
    let on_progress: fn(u64, u64) = |_, _| {};
    BlobCursor::new(SliceSource::new(bytes), bytes.len() as u64, on_progress).with_chunk_size(chunk)
}

/// Scans `json` as a column document with `chunk`-byte chunks.
fn columns(json: &str, chunk: usize) -> Result<Vec<(String, u64)>, String> {
    let mut cur = cursor(json.as_bytes(), chunk);
    block_on(scan_columns(&mut cur)).map(|cols| {
        cols.into_iter()
            .map(|c| (c.key, c.count))
            .collect::<Vec<_>>()
    })
}

/// The error `columns` produced, for asserting on rather than unwrapping.
fn error_of(json: &str, chunk: usize) -> Option<String> {
    columns(json, chunk).err()
}

// ── the grammar ──────────────────────────────────────────────────────────

#[test]
fn one_column_per_top_level_key() {
    assert_eq!(
        columns(r#"{"a":1,"b":2,"c":3}"#, 64),
        Ok(vec![
            ("a".to_string(), 1),
            ("b".to_string(), 1),
            ("c".to_string(), 1)
        ]),
        "one entry per key, in document order"
    );
}

/// A nested object's key counts as a leaf, so `{"y":3}` is 2.
///
/// Both implementations of this grammar do this, which is why it is pinned
/// as intended rather than as a discrepancy. The claim on this test used to
/// be that `count_non_null_leaves` gave 1 here and disagreed with the
/// streaming scanner; a differential run over both showed they agree on
/// every nested-key input. The one input where they really did differ was
/// the empty string, and that is fixed.
#[test]
fn a_nested_objects_key_is_counted_as_a_leaf() {
    assert_eq!(
        columns(r#"{"x":[1,null,2,{"y":3},null]}"#, 64),
        Ok(vec![("x".to_string(), 4)]),
        r#"1 + 2 + the key "y" + 3, with both nulls counting 0"#
    );
}

#[test]
fn nulls_count_nothing_and_everything_else_counts_one() {
    assert_eq!(
        columns(r#"{"x":[1,null,2,null,3]}"#, 64),
        Ok(vec![("x".to_string(), 3)]),
        "a column of five rows with two nulls is three"
    );
}

/// An empty string is present but holds nothing, so it is not a value.
///
/// This is the rule that chose between the two implementations. It reads as
/// "1 if non-empty" on `count_value` and was applied there and not in
/// `scan_container`, so a column whose nested rows had an empty key or an
/// empty string value was over-reported by one per empty string. Those
/// numbers move with this fix, which is the point of pinning them.
#[test]
fn an_empty_string_counts_nothing_wherever_it_appears() {
    for (json, expected, why) in [
        (r#"{"x":[""]}"#, 0, "a lone empty string is not a leaf"),
        (r#"{"x":["","a"]}"#, 1, "one of two strings is empty"),
        (
            r#"{"x":[{"":1}]}"#,
            1,
            "the empty key is not a leaf, the 1 is",
        ),
        (
            r#"{"x":[{"y":""}]}"#,
            1,
            "the key counts, the empty value does not",
        ),
        (
            r#"{"x":[{"y":""},{"z":1}]}"#,
            3,
            "one empty string and two populated",
        ),
        (
            r#"{"x":["\\"]}"#,
            1,
            "a lone backslash is an escape, so it is not empty",
        ),
    ] {
        assert_eq!(
            columns(json, 64),
            Ok(vec![("x".to_string(), expected)]),
            "{why}: {json}"
        );
    }
}

#[test]
fn an_empty_object_has_no_columns() {
    assert_eq!(columns("{}", 64), Ok(Vec::new()));
}

// ── chunk boundaries, which is what the source seam is for ──────────────

/// The reader's one real obligation: a token means the same thing whether it
/// falls inside a chunk or straddles two.
///
/// Swept at every chunk size from 1 upward, so every byte offset in the
/// document is a boundary somewhere in the sweep. This is the test that found
/// all three of the boundary bugs below, each of which hangs or corrupts the
/// result at one specific chunk size and looks correct at every other.
#[test]
fn the_result_does_not_depend_on_where_the_chunks_fall() {
    const DOC: &str = r#"{"str":"a\"b\\c","arr":[1,[2,{"k":"v"}]],"n":null,"t":true}"#;
    let whole = columns(DOC, DOC.len());
    for chunk in 1..=DOC.len() {
        assert_eq!(
            columns(DOC, chunk),
            whole,
            "chunk size {chunk} changed the answer; the reader is \
             boundary-dependent"
        );
    }
}

/// The three bugs this module's first run turned up, each pinned at the chunk
/// size that triggers it. They are listed together because they are one
/// mistake repeated — `read_json_key` and `skip_string_nonempty` implement the
/// same scan, and only one of them was right about where the cursor ends up.
#[test]
fn a_key_split_across_a_chunk_boundary_is_one_key() {
    // Every third byte is a boundary. `read_json_key` refilled without
    // draining, so each chunk re-scanned the last one's bytes: this key came
    // back as "sststrstr".
    assert_eq!(
        columns(r#"{"keyname":1}"#, 3),
        Ok(vec![("keyname".to_string(), 1)]),
        "each chunk contributes its own bytes exactly once"
    );
}

#[test]
fn a_long_key_spanning_many_chunks_is_not_repeated() {
    // The same defect with a key long enough to span many chunks, where the
    // unread `raw` buffer grows on every one of them.
    let key = "k".repeat(200);
    let json = format!(r#"{{"{key}":1}}"#);
    assert_eq!(
        columns(&json, 4),
        Ok(vec![(key, 1)]),
        "the key is read once, not once per chunk it spans"
    );
}

#[test]
fn a_number_split_across_a_chunk_boundary_is_one_number() {
    assert_eq!(
        columns(r#"{"n":123456789}"#, 3),
        Ok(vec![("n".to_string(), 1)]),
        "digits split three ways are still one leaf"
    );
}

// ── the escape arm mutants delete most ──────────────────────────────────

/// The `b'\\'` arm in `skip_string_nonempty` decides whether the next byte is
/// a literal, and `delete` on it is among the most common survivors found
/// here. Removing it makes `a"b` scan as `a` plus a syntax error.
#[test]
fn an_escaped_quote_does_not_end_the_string() {
    assert_eq!(
        columns(r#"{"k":"a\"b"}"#, 64),
        Ok(vec![("k".to_string(), 1)]),
        r#""a\"b" is one string of three characters, not a then an error"#
    );
}

/// A body ending in a lone backslash: the escape flag is set and the stream
/// ends with no closing quote. Truncated multi-gigabyte uploads are ordinary,
/// not adversarial, so this must terminate with an error rather than loop
/// looking for the quote a whole file would have had.
#[test]
fn a_body_ending_in_a_lone_backslash_is_an_error_and_not_a_hang() {
    assert!(
        error_of(r#"{"k":"a\"#, 64).is_some_and(|e| e.contains("EOF")),
        "expected an EOF error, got {:?}",
        error_of(r#"{"k":"a\"#, 64)
    );
}

#[test]
fn an_unterminated_string_is_an_error() {
    assert!(
        error_of(r#"{"k":"abc"#, 64).is_some_and(|e| e.contains("EOF")),
        "expected an EOF error, got {:?}",
        error_of(r#"{"k":"abc"#, 64)
    );
}

// ── malformed input is reported, not guessed at ──────────────────────────

#[test]
fn a_document_that_is_not_an_object_is_rejected() {
    assert_eq!(
        columns("[1,2,3]", 64),
        Err("Expected a top-level JSON object".to_string())
    );
}

#[test]
fn a_missing_colon_is_rejected() {
    assert_eq!(
        columns(r#"{"k" 1}"#, 64),
        Err("Expected ':' after object key".to_string())
    );
}

#[test]
fn a_trailing_comma_does_not_invent_a_column() {
    // The loop breaks on a `,` and then finds EOF. It must stop, not loop
    // asking for a key that is not there, and it must not report an empty
    // column as a real one.
    assert_eq!(
        columns(r#"{"a":1,}"#, 64),
        Ok(vec![("a".to_string(), 1)]),
        "one column read, and the trailing comma added none"
    );
}

/// The two implementations of this grammar, on the same value.
///
/// This is the equivalence that actually exists: `count_non_null_leaves`
/// counts the leaves of one JSON value and so does `count_value`. At a
/// four-byte chunk the streaming side crosses boundaries throughout.
///
/// Every empty-string case is in here on purpose. That was the only rule
/// the two ever disagreed on, and the list above had no empty string
/// anywhere in it — not as a key, not as a value, not nested — so the
/// cross-check passed for the whole time the two were returning different
/// answers for `[{"":1}]`. A disagreement test only covers the inputs it
/// lists; this one was listing the wrong ones.
#[test]
fn the_streaming_value_counter_matches_the_in_memory_one() {
    for value in [
        "1",
        "null",
        "true",
        "false",
        r#""x""#,
        r#""""#,
        r#""a\"b""#,
        "[]",
        "{}",
        "[1,2,3]",
        "[1,null,2]",
        r#"[{"y":3}]"#,
        r#"{"a":null,"b":2}"#,
        r#"[[1,[2]],{"k":"v"}]"#,
        // Empty strings, in every position they can appear: as a nested
        // key, as a nested value, as both, and inside an empty container.
        r#"[{"":1}]"#,
        r#"{"a":{"":1}}"#,
        r#"{"":1,"b":2}"#,
        r#"{"a":1,"":2}"#,
        r#"[{"":""}]"#,
        r#"[{"":null}]"#,
        r#"{"":{}}"#,
        r#"[{"":[]}]"#,
        r#"{"x":[{"":1}]}"#,
        r#"[{"y":""}]"#,
        r#"[[{"":1}]]"#,
        r#"[{"":1},{"":2}]"#,
        // A backslash makes a string non-empty even when the escaped byte
        // is the quote that looks like the end of it.
        r#"["\\"]"#,
    ] {
        let mut cur = cursor(value.as_bytes(), 4);
        let streamed = block_on(super::count_value(&mut cur));
        assert_eq!(
            streamed.as_ref().copied().ok(),
            Some(super::count_non_null_leaves(value)),
            "the two implementations disagree on {value:?}: {streamed:?}"
        );
    }
}
