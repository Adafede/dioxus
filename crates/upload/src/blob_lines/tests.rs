// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

// Tests for `BlobLines`, over a `SliceSource` so they run on the host.
//
// This type had none, and it is the reader behind the file input of two of the
// six apps -- `lipid-selecto-rs` and `mgf-precursor-erro-rs`. It shipped a spin
// loop at end-of-stream that neither could be caught by, because the apps were
// the only exercisers and a browser was the only place the bug could run.

use super::BlobLines;
use crate::bytes::SliceSource;
use futures_executor::block_on;

/// Reads every line out of `text`, then stops.
///
/// The `take` cap is what turns a non-terminating reader into a failing test
/// rather than a hanging one: a reader that keeps handing back `Some("")`
/// reaches 64 and returns, and the assertion below says what was collected.
fn drain(text: &str, chunk: usize) -> Vec<String> {
    block_on(async {
        let reader =
            BlobLines::new(SliceSource::new(text.as_bytes()), |_, _| {}).with_chunk_size(chunk);
        let mut reader = reader;
        let mut lines = Vec::new();
        while lines.len() < 64 {
            match reader.next_line().await {
                Ok(Some(line)) => lines.push(line),
                Ok(None) | Err(_) => break,
            }
        }
        lines
    })
}

#[test]
fn reads_every_line_of_a_file_that_ends_in_a_newline() {
    assert_eq!(drain("a\nb\nc\n", 64), ["a", "b", "c"]);
}

/// The bug: past the last line this used to yield `Some("")` for ever, so the
/// caller collected thousands of empty strings until `Vec`'s capacity
/// computation overflowed a 32-bit `usize` — which is what a browser showed as
/// `capacity overflow`, and what a host build never showed at all.
#[test]
fn stops_at_end_of_file_instead_of_returning_empty_lines() {
    assert_eq!(
        drain("a\nb\n", 64),
        ["a", "b"],
        "must not yield an empty tail"
    );
}

/// Asking again after `None` must keep answering `None`. The spin needed only
/// one caller that kept asking, but a caller that treats `None` as final and
/// then polls must not resurrect it either.
#[test]
fn stays_terminated_once_it_has_ended() {
    block_on(async {
        let mut reader = BlobLines::new(SliceSource::new(b"only\n"), |_, _| {}).with_chunk_size(64);
        assert_eq!(reader.next_line().await.unwrap().as_deref(), Some("only"));
        for _ in 0..8 {
            assert!(
                reader.next_line().await.unwrap().is_none(),
                "a terminated reader must stay terminated"
            );
        }
    });
}

/// The last line of a file with no trailing newline is still a line, and is
/// still returned exactly once.
#[test]
fn returns_the_unterminated_final_line_once() {
    assert_eq!(drain("a\nb", 64), ["a", "b"]);
    assert_eq!(drain("a\nb\ntrailing", 64), ["a", "b", "trailing"]);
}

/// A blank line inside the file is a line and must be kept; only the *tail* is
/// not a line.
#[test]
fn keeps_blank_lines_that_are_actually_in_the_file() {
    assert_eq!(drain("a\n\nb\n", 64), ["a", "", "b"]);
    assert_eq!(drain("\n\n", 64), ["", ""]);
    assert!(
        drain("", 64).is_empty(),
        "an empty file has no lines at all"
    );
}

/// At the default 16 MiB no test can put a line across a chunk boundary, which
/// is what `with_chunk_size` is for. These use a chunk smaller than the input.
#[test]
fn a_line_straddling_a_chunk_boundary_comes_back_whole() {
    assert_eq!(drain("alpha\nbeta\ngamma\n", 4), ["alpha", "beta", "gamma"]);
    assert_eq!(drain("alpha\nbeta\ngamma\n", 1), ["alpha", "beta", "gamma"]);
}

/// `\r\n` is stripped to the bare `\r`-less line, at any chunking.
#[test]
fn strips_carriage_returns() {
    assert_eq!(drain("a\r\nb\r\n", 64), ["a", "b"]);
    assert_eq!(drain("a\r\nb\r\n", 3), ["a", "b"]);
}

/// A UTF-8 sequence split across a chunk boundary must survive the split.
#[test]
fn keeps_multibyte_characters_intact_across_a_boundary() {
    assert_eq!(drain("résumé\nαβγ\n", 2), ["résumé", "αβγ"]);
}
