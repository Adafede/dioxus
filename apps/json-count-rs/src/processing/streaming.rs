// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

// The `streaming` tests, extracted from `processing.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `processing` module, so
// `use super::*` below reaches exactly what it did before the move.
use futures_executor::block_on;
use upload::{BlobCursor, SliceSource};

use super::{progress_message, scan_columns, scan_source_with_progress};

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

/// The same escape, with something after it for the mistake to swallow.
///
/// Every mutant of the escape arms in `skip_string_nonempty` — the `b'\\'`
/// arm deleted, the `escaped` guard forced to `false`, the `!escaped` guard
/// forced to `true` — makes the escaped quote end the string early. On
/// `"a\"b"` alone that is invisible: the count is 1 either way, because a
/// non-empty string counts one whether it is three characters or one, and the
/// `b` left over becomes a bare scalar that `scan_columns` then trips over and
/// stops on. Two columns is what it takes — the trailing one is missing under
/// the mutation and present without it.
#[test]
fn an_escaped_quote_does_not_eat_the_columns_after_it() {
    for (json, expected, why) in [
        (
            r#"{"k":"a\"b","z":2}"#,
            vec![("k".to_string(), 1), ("z".to_string(), 1)],
            "an escaped quote inside the first value",
        ),
        (
            r#"{"k":"\"","z":2}"#,
            vec![("k".to_string(), 1), ("z".to_string(), 1)],
            "a string that is nothing but an escaped quote",
        ),
        (
            r#"{"k":"a\\","z":2}"#,
            vec![("k".to_string(), 1), ("z".to_string(), 1)],
            "an escaped backslash: the quote after it really does close",
        ),
        (
            r#"{"k":"a\"b","z":"c\"d","y":null}"#,
            vec![
                ("k".to_string(), 1),
                ("z".to_string(), 1),
                ("y".to_string(), 0),
            ],
            "two values with an escaped quote each",
        ),
    ] {
        assert_eq!(columns(json, 64), Ok(expected), "{why}: {json}");
    }
}

/// The same three mutants in `read_json_key`, on the key rather than the value.
///
/// A key is read with its escapes kept and its body handed to
/// `unescape_json_string`, so `"a\"b"` is the two-character key `a"b`. Any of
/// the three mutations ends the key at the escaped quote, leaves the rest of it
/// in the stream, and the colon check then reads a letter rather than a colon —
/// which is an error, and an error the test below does not expect.
#[test]
fn an_escaped_quote_in_a_key_is_part_of_the_key() {
    for (json, expected, why) in [
        (r#"{"a\"b":1}"#, vec![("a\"b", 1)], "an escaped quote"),
        (r#"{"a\\b":1}"#, vec![("a\\b", 1)], "an escaped backslash"),
        (r#"{"a\u0041b":1}"#, vec![("aAb", 1)], "a unicode escape"),
        (
            r#"{"a\"b":1,"c":2}"#,
            vec![("a\"b", 1), ("c", 1)],
            "an escaped quote, with a second column",
        ),
    ] {
        assert_eq!(
            columns(json, 64),
            Ok(expected
                .into_iter()
                .map(|(k, c)| (k.to_string(), c))
                .collect()),
            "{why}: {json}"
        );
    }
}

/// The key escapes above, swept over every chunk size like the value ones.
///
/// A key split across a boundary has to keep its escape flag between passes,
/// and that state is carried in the same variable as everything else.
#[test]
fn an_escaped_quote_in_a_key_survives_every_chunk_boundary() {
    const DOC: &str = r#"{"a\"b":1,"c\\d":2}"#;
    let whole = columns(DOC, DOC.len());
    for chunk in 1..=DOC.len() {
        assert_eq!(
            columns(DOC, chunk),
            whole,
            "chunk size {chunk} changed the answer; the reader is boundary-dependent"
        );
    }
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

// ── the browser shell, with the browser taken out ──────────────────────────

/// The progress line, and the three arithmetic decisions in it.
///
/// This was inside the callback the browser passed to the scanner, which is a
/// closure in a function that takes a `Blob`. Nothing could reach it. Each of
/// these is a mutant: `total.max(1)` losing its `max` divides by zero on an
/// empty upload, `processed.min(safe_total)` losing its `min` shows a count past
/// the end, and the `.min(100)` losing its clamp shows 100% while it is not.
#[test]
fn the_progress_line_is_clamped_at_both_ends() {
    assert_eq!(
        progress_message(0, 1000),
        "Scanning 0/1000 bytes (0%)...",
        "nothing read yet"
    );
    assert_eq!(
        progress_message(500, 1000),
        "Scanning 500/1000 bytes (50%)...",
        "halfway"
    );
    assert_eq!(
        progress_message(1000, 1000),
        "Scanning 1000/1000 bytes (100%)...",
        "the whole of it"
    );
    assert_eq!(
        progress_message(1500, 1000),
        "Scanning 1000/1000 bytes (100%)...",
        "a count past the total is shown as the total, not as 150%"
    );
    assert_eq!(
        progress_message(1001, 1000),
        "Scanning 1000/1000 bytes (100%)...",
        "and the percentage stops at 100 rather than running past it"
    );
}

/// An empty upload has a total of zero, and the guard is what keeps the
/// percentage from dividing by it.
#[test]
fn an_empty_upload_reports_zero_of_zero_and_not_a_division() {
    assert_eq!(
        progress_message(0, 0),
        "Scanning 0/1 bytes (0%)...",
        "the denominator is one when the total is zero"
    );
}

/// The browser scanner with the browser removed: a slice where a `Blob` was.
///
/// Same call, same columns, and the only thing that changed is that a host test
/// can make it — which is the point of [`super::scan_source_with_progress`]
/// existing separately from the entry point that wraps it.
#[test]
fn the_browser_scanner_is_the_slice_scanner() {
    let on_progress: fn(u64, u64) = |_, _| {};
    let json = r#"{"a":1,"b":[2,null,3]}"#;
    let scanned = block_on(scan_source_with_progress(
        SliceSource::new(json.as_bytes()),
        json.len() as u64,
        on_progress,
    ));
    assert_eq!(
        scanned.map(|cols| {
            cols.into_iter()
                .map(|c| (c.key, c.count))
                .collect::<Vec<_>>()
        }),
        Ok(vec![("a".to_string(), 1), ("b".to_string(), 2)]),
        "the same columns the in-browser scanner reports"
    );
    assert_eq!(
        block_on(scan_source_with_progress(
            SliceSource::new(b"not an object"),
            12,
            on_progress
        )),
        Err("Expected a top-level JSON object".to_string()),
        "and the same refusal"
    );
}
