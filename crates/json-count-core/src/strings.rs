// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! Streaming JSON string scanners for keys and value-skipping.

use stream_io::{BlobCursor, ChunkSource};

use crate::container::drive_cursor_chunks;
use crate::error::JsonCountError;
use crate::unescape::{StringValidator, unescape_validated_json_string_into};

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
    let mut i = 0usize;
    while let Some(rest) = tail.get(i..) {
        if rest.is_empty() {
            break;
        }
        let run = validator.plain_ascii_run(rest);
        if run > 0 {
            let remaining_cap = MAX_KEY_BYTES.saturating_sub(raw.len());
            if run > remaining_cap {
                return Err(JsonCountError::KeyTooLong);
            }
            if let Some(chunk) = rest.get(..run) {
                raw.extend_from_slice(chunk);
            }
            i += run;
        }
        let Some(&byte) = tail.get(i) else {
            break;
        };
        i += 1;
        if validator.step(byte)? {
            return Ok((i, true));
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
    drive_cursor_chunks(cursor, |tail| {
        let (consumed, closed) = scan_key_chunk(tail, raw, &mut validator)?;
        if closed {
            unescape_validated_json_string_into(raw, validator.has_backslash(), out)?;
            Ok((consumed, Some(())))
        } else {
            Ok((consumed, None))
        }
    })
    .await?
    .ok_or(JsonCountError::UnexpectedEof("while reading string"))
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
    let mut i = 0usize;
    while let Some(rest) = tail.get(i..) {
        if rest.is_empty() {
            break;
        }
        let run = validator.plain_ascii_run(rest);
        if run > 0 {
            has_body = true;
            i += run;
        }
        let Some(&byte) = tail.get(i) else {
            break;
        };
        i += 1;
        if validator.step(byte)? {
            return Ok((i, has_body, true));
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
    drive_cursor_chunks(cursor, |tail| {
        let (consumed, has_body, closed) = scan_skip_chunk(tail, &mut validator)?;
        if has_body {
            any = true;
        }
        Ok((consumed, closed.then_some(any)))
    })
    .await?
    .ok_or(JsonCountError::UnexpectedEof("while reading string"))
}
