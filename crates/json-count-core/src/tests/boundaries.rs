// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

use std::cell::RefCell;
use std::rc::Rc;

use futures_executor::block_on;
use stream_io::{BlobCursor, SliceSource};

use super::{columns, cursor};
use crate::scanner::scan_columns;
use crate::strings::{read_json_key, skip_string_nonempty};
use crate::{JsonCountError, progress_message, scan_source_with_progress};

#[test]
fn the_result_does_not_depend_on_where_the_chunks_fall() {
    const DOC: &str = r#"{"str":"a\"b\\c","arr":[1,[2,{"k":"v"}]],"n":null,"t":true}"#;
    let whole = columns(DOC, DOC.len());
    for chunk in 1..=DOC.len() {
        assert_eq!(
            columns(DOC, chunk),
            whole,
            "chunk size {chunk} changed the answer"
        );
    }
}

#[test]
fn a_key_split_across_a_chunk_boundary_is_one_key() {
    assert_eq!(
        columns(r#"{"keyname":1}"#, 3),
        Ok(vec![("keyname".to_string(), 1)]),
        "each chunk contributes its own bytes once"
    );
    let key = "k".repeat(200);
    let json = format!(r#"{{"{key}":1}}"#);
    assert_eq!(
        columns(&json, 4),
        Ok(vec![(key, 1)]),
        "long key spans many chunks without duplication"
    );
}

#[test]
fn surrogate_pair_and_escaped_keys_survive_every_chunk_boundary() {
    const DOC: &str = r#"{"a\"b":1,"c\\d":2,"e\ud83d\ude00f":3}"#;
    let whole = columns(DOC, DOC.len());
    assert_eq!(
        whole,
        Ok(vec![
            ("a\"b".to_string(), 1),
            ("c\\d".to_string(), 1),
            ("e😀f".to_string(), 1),
        ]),
        "full-buffer surrogate pair and escaped keys parse accurately"
    );
    for chunk in [1, 2, 3, 5, 7, 13, 64] {
        assert_eq!(
            columns(DOC, chunk),
            whole,
            "chunk size {chunk} must yield identical escaped/surrogate keys"
        );
    }
}

#[test]
fn read_json_key_and_skip_string_nonempty_validate_quotes_and_eof() {
    let mut cur = cursor(b"not_quoted", 4);
    assert_eq!(
        block_on(read_json_key(&mut cur)),
        Err(JsonCountError::ExpectedOpeningQuote),
        "read_json_key requires opening quote"
    );

    let mut cur2 = cursor(b"not_quoted", 4);
    assert_eq!(
        block_on(skip_string_nonempty(&mut cur2)),
        Err(JsonCountError::ExpectedOpeningQuote),
        "skip_string_nonempty requires opening quote"
    );

    let mut cur3 = cursor(b"\"abc", 2);
    assert!(
        matches!(
            block_on(skip_string_nonempty(&mut cur3)),
            Err(JsonCountError::UnexpectedEof(_))
        ),
        "skip_string_nonempty reports EOF on unclosed string"
    );
}

#[test]
fn progress_message_and_callback_invocation() {
    assert_eq!(
        progress_message(0, 1000),
        "Scanning 0/1000 bytes (0%)...",
        "0%"
    );
    assert_eq!(
        progress_message(500, 1000),
        "Scanning 500/1000 bytes (50%)...",
        "50%"
    );
    assert_eq!(
        progress_message(1500, 1000),
        "Scanning 1000/1000 bytes (100%)...",
        "clamped 100%"
    );
    assert_eq!(
        progress_message(0, 0),
        "Scanning 0/0 bytes (0%)...",
        "zero-byte upload guard"
    );
    assert_eq!(
        progress_message(1_u64 << 62, 1_u64 << 63),
        format!("Scanning {}/{} bytes (50%)...", 1_u64 << 62, 1_u64 << 63),
        "large byte counts do not overflow percentage math"
    );

    let events = Rc::new(RefCell::new(Vec::<(u64, u64)>::new()));
    let sink = Rc::clone(&events);
    let doc = format!(r#"{{"k":"{}"}}"#, "a".repeat(5 * 1024 * 1024));
    let mut cur = BlobCursor::new(
        SliceSource::new(doc.as_bytes()),
        doc.len() as u64,
        move |p, t| sink.borrow_mut().push((p, t)),
    )
    .with_chunk_size(1_048_576);
    let res = block_on(scan_columns(&mut cur));
    assert!(res.is_ok(), "chunked scan should succeed");
    let recorded = events.borrow();
    assert!(
        !recorded.is_empty(),
        "progress callback must fire on multi-megabyte stream"
    );
    assert!(
        recorded.is_sorted_by_key(|&(p, _)| p),
        "progress byte counts must be monotonic: {recorded:?}"
    );
    assert!(
        recorded
            .last()
            .copied()
            .is_some_and(|(p, t)| p >= 4 * 1024 * 1024 && p <= t && t == doc.len() as u64),
        "progress event must cross 4 MiB threshold with matching total: {recorded:?}"
    );

    let on_progress: fn(u64, u64) = |_, _| {};
    let err = block_on(scan_source_with_progress(
        SliceSource::new(b"not an object"),
        13,
        on_progress,
    ));
    assert_eq!(
        err,
        Err(JsonCountError::ExpectedTopLevelObject),
        "non-object rejected"
    );
}

#[test]
fn key_length_cap_accepts_exact_limit_and_rejects_one_over() {
    let max_plain = format!("\"{}\"", "k".repeat(1_048_576));
    let mut cur_ok = cursor(max_plain.as_bytes(), 65_536);
    assert!(
        block_on(read_json_key(&mut cur_ok)).is_ok(),
        "exact MAX_KEY_BYTES key should succeed"
    );

    let over_plain = format!("\"{}\"", "k".repeat(1_048_577));
    let mut cur_over = cursor(over_plain.as_bytes(), 65_536);
    assert_eq!(
        block_on(read_json_key(&mut cur_over)),
        Err(JsonCountError::KeyTooLong),
        "one byte over MAX_KEY_BYTES should fail"
    );

    let exact_escaped = format!("\"{}\\n\"", "k".repeat(1_048_574));
    let mut cur_esc_ok = cursor(exact_escaped.as_bytes(), 1_048_576);
    assert!(
        block_on(read_json_key(&mut cur_esc_ok)).is_ok(),
        "escaped key hitting exact MAX_KEY_BYTES should succeed"
    );

    let over_escaped = format!("\"{}\\n\"", "k".repeat(1_048_575));
    let mut cur_esc_over = cursor(over_escaped.as_bytes(), 1_048_576);
    assert_eq!(
        block_on(read_json_key(&mut cur_esc_over)),
        Err(JsonCountError::KeyTooLong),
        "escaped key exceeding MAX_KEY_BYTES should fail"
    );
}
