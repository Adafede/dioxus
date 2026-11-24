// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! Streaming JSON string scanners for keys and value-skipping.

use stream_io::{BlobCursor, ChunkSource};

use crate::error::JsonCountError;
use crate::unescape::unescape_json_string_into;

const MAX_KEY_BYTES: usize = 1_048_576;

pub(crate) fn pass_bound<F: FnMut(u64, u64), S: ChunkSource>(cursor: &BlobCursor<F, S>) -> usize {
    usize::try_from(cursor.total_bytes())
        .unwrap_or(usize::MAX)
        .saturating_add(1)
}

fn push_key_slice(raw: &mut Vec<u8>, slice: &[u8]) -> Result<(), JsonCountError> {
    if raw.len().saturating_add(slice.len()) > MAX_KEY_BYTES {
        return Err(JsonCountError::KeyTooLong);
    }
    raw.extend_from_slice(slice);
    Ok(())
}

fn scan_key_chunk(
    tail: &[u8],
    raw: &mut Vec<u8>,
    escaped: &mut bool,
) -> Result<(usize, bool), JsonCountError> {
    let mut pos = 0usize;
    while let Some((&first, _)) = tail.get(pos..).and_then(<[u8]>::split_first) {
        if *escaped {
            *escaped = false;
            push_key_slice(raw, &[b'\\', first])?;
            pos += 1;
            continue;
        }
        let rest = tail.get(pos..).unwrap_or_default();
        let next_special = rest.iter().position(|&b| b == b'"' || b == b'\\');
        let run_len = next_special.unwrap_or(rest.len());
        if let Some(plain) = rest.get(..run_len) {
            push_key_slice(raw, plain)?;
        }
        let Some(&delim) = next_special.and_then(|offset| rest.get(offset)) else {
            return Ok((tail.len(), false));
        };
        pos += run_len + 1;
        if delim == b'"' {
            return Ok((pos, true));
        }
        *escaped = true;
    }
    Ok((pos, false))
}

/// Reads and unescapes a JSON string key into `out`, reusing `raw` and `out` buffers.
pub(crate) async fn read_json_key_into<F: FnMut(u64, u64), S: ChunkSource>(
    cursor: &mut BlobCursor<F, S>,
    raw: &mut Vec<u8>,
    out: &mut String,
) -> Result<(), JsonCountError> {
    if cursor.next_byte().await? != Some(b'"') {
        return Err(JsonCountError::ExpectedOpeningQuote);
    }

    raw.clear();
    let mut escaped = false;
    for _ in 0..pass_bound(cursor) {
        let Some(tail) = cursor.buffer().get(cursor.pos()..) else {
            break;
        };
        let (consumed, closed) = scan_key_chunk(tail, raw, &mut escaped)?;
        cursor.advance(consumed);
        if closed {
            unescape_json_string_into(raw, out);
            return Ok(());
        }
        if !cursor.fill().await? {
            return Err(JsonCountError::UnexpectedEof("while reading string"));
        }
    }

    Err(JsonCountError::Unterminated("String key"))
}

/// Reads a JSON string key from `cursor`.
///
/// # Errors
///
/// Returns [`JsonCountError`] on malformed string syntax, oversized keys, or I/O errors.
#[cfg(test)]
pub(crate) async fn read_json_key<F: FnMut(u64, u64), S: ChunkSource>(
    cursor: &mut BlobCursor<F, S>,
) -> Result<String, JsonCountError> {
    let mut raw = Vec::new();
    let mut out = String::new();
    read_json_key_into(cursor, &mut raw, &mut out).await?;
    Ok(out)
}

fn scan_skip_chunk(tail: &[u8], escaped: &mut bool) -> (usize, usize, bool) {
    let mut consumed = 0usize;
    let mut body = 0usize;
    let mut closed = false;
    for (i, &byte) in tail.iter().enumerate() {
        consumed = i + 1;
        match byte {
            _ if *escaped => *escaped = false,
            b'\\' => *escaped = true,
            b'"' => {
                closed = true;
                break;
            }
            _ => {}
        }
        body = consumed;
    }
    (consumed, body, closed)
}

/// Skips over a JSON string value and reports whether it was non-empty.
///
/// # Errors
///
/// Returns [`JsonCountError`] on missing opening quote, EOF inside string, or I/O failure.
pub(crate) async fn skip_string_nonempty<F: FnMut(u64, u64), S: ChunkSource>(
    cursor: &mut BlobCursor<F, S>,
) -> Result<bool, JsonCountError> {
    if cursor.next_byte().await? != Some(b'"') {
        return Err(JsonCountError::ExpectedOpeningQuote);
    }

    let mut escaped = false;
    let mut any = false;
    for _ in 0..pass_bound(cursor) {
        let Some(tail) = cursor.buffer().get(cursor.pos()..) else {
            break;
        };
        let (consumed, body, closed) = scan_skip_chunk(tail, &mut escaped);
        if body > 0 {
            any = true;
        }
        cursor.advance(consumed);
        if closed {
            return Ok(any);
        }
        if !cursor.fill().await? {
            return Err(JsonCountError::UnexpectedEof("while reading string"));
        }
    }

    Err(JsonCountError::Unterminated("String"))
}
