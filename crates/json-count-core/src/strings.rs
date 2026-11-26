// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! Streaming JSON string scanners for keys and value-skipping.

use stream_io::{BlobCursor, ChunkSource};

use crate::error::JsonCountError;
use crate::unescape::{StringValidator, unescape_json_string_into};

const MAX_KEY_BYTES: usize = 1_048_576;

fn push_key_byte(raw: &mut Vec<u8>, byte: u8) -> Result<(), JsonCountError> {
    if raw.len() >= MAX_KEY_BYTES {
        return Err(JsonCountError::KeyTooLong);
    }
    raw.push(byte);
    Ok(())
}

fn scan_key_chunk(
    tail: &[u8],
    raw: &mut Vec<u8>,
    validator: &mut StringValidator,
) -> Result<(usize, bool), JsonCountError> {
    for (i, &byte) in tail.iter().enumerate() {
        if validator.step(byte)? {
            return Ok((i + 1, true));
        }
        push_key_byte(raw, byte)?;
    }
    Ok((tail.len(), false))
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
    let mut validator = StringValidator::default();
    loop {
        let tail = cursor.buffer().get(cursor.pos()..).unwrap_or_default();
        let (consumed, closed) = scan_key_chunk(tail, raw, &mut validator)?;
        cursor.advance(consumed);
        if closed {
            return unescape_json_string_into(raw, out);
        }
        if !cursor.fill().await? {
            return Err(JsonCountError::UnexpectedEof("while reading string"));
        }
    }
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

fn scan_skip_chunk(
    tail: &[u8],
    validator: &mut StringValidator,
) -> Result<(usize, bool, bool), JsonCountError> {
    let mut has_body = false;
    for (i, &byte) in tail.iter().enumerate() {
        if validator.step(byte)? {
            return Ok((i + 1, has_body, true));
        }
        has_body = true;
    }
    Ok((tail.len(), has_body, false))
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

    let mut validator = StringValidator::default();
    let mut any = false;
    loop {
        let tail = cursor.buffer().get(cursor.pos()..).unwrap_or_default();
        let (consumed, has_body, closed) = scan_skip_chunk(tail, &mut validator)?;
        if has_body {
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
}
