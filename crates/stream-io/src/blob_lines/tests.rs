// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

// Tests for `BlobLines`, over a `SliceSource` so they run on the host.

use super::BlobLines;
use crate::StreamError;
use crate::bytes::SliceSource;
use futures_executor::block_on;

fn drain(text: &str, chunk: usize) -> Vec<String> {
    block_on(async {
        let mut reader =
            BlobLines::new(SliceSource::new(text.as_bytes()), |_, _| {}).with_chunk_size(chunk);
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

#[test]
fn stops_at_end_of_file_instead_of_returning_empty_lines() {
    assert_eq!(
        drain("a\nb\n", 64),
        ["a", "b"],
        "must not yield an empty tail"
    );
}

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

#[test]
fn returns_the_unterminated_final_line_once() {
    assert_eq!(drain("a\nb", 64), ["a", "b"]);
    assert_eq!(drain("a\nb\ntrailing", 64), ["a", "b", "trailing"]);
}

#[test]
fn keeps_blank_lines_that_are_actually_in_the_file() {
    assert_eq!(drain("a\n\nb\n", 64), ["a", "", "b"]);
    assert_eq!(drain("\n\n", 64), ["", ""]);
    assert!(
        drain("", 64).is_empty(),
        "an empty file has no lines at all"
    );
}

#[test]
fn a_line_straddling_a_chunk_boundary_comes_back_whole() {
    assert_eq!(drain("alpha\nbeta\ngamma\n", 4), ["alpha", "beta", "gamma"]);
    assert_eq!(drain("alpha\nbeta\ngamma\n", 1), ["alpha", "beta", "gamma"]);
}

#[test]
fn strips_carriage_returns() {
    assert_eq!(drain("a\r\nb\r\n", 64), ["a", "b"]);
    assert_eq!(drain("a\r\nb\r\n", 3), ["a", "b"]);
}

#[test]
fn keeps_multibyte_characters_intact_across_a_boundary() {
    assert_eq!(drain("résumé\nαβγ\n", 2), ["résumé", "αβγ"]);
}

#[test]
fn accepts_line_up_to_max_line_bytes_and_rejects_oversized_lines() {
    block_on(async {
        let mut ok_reader = BlobLines::new(SliceSource::new(b"12345\n12345\r\n"), |_, _| {})
            .with_chunk_size(2)
            .with_max_line_bytes(5);
        assert_eq!(
            ok_reader.next_line().await.unwrap().as_deref(),
            Some("12345")
        );
        assert_eq!(
            ok_reader.next_line().await.unwrap().as_deref(),
            Some("12345")
        );
        assert!(ok_reader.next_line().await.unwrap().is_none());

        for (input, chunk) in [("123456\n", 2), ("123456\n", 64), ("123456", 2)] {
            let mut reader = BlobLines::new(SliceSource::new(input.as_bytes()), |_, _| {})
                .with_chunk_size(chunk)
                .with_max_line_bytes(5);
            let err = reader.next_line().await.unwrap_err();
            assert!(
                matches!(err, StreamError::LineTooLong { max_bytes: 5 }),
                "expected LineTooLong for ({input:?}, chunk={chunk}), got {err:?}",
            );
        }
    });
}
