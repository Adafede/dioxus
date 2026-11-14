// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! Streaming JSON field scanner.
//!
//! Counts non-null values per top-level key of an uploaded JSON object using a
//! streaming `BlobCursor`, keeping memory bounded for multi-gigabyte files.
//!
//! # Two implementations of one grammar
//!
//! Two, and only the second one ships: `count_value` is the streaming scanner the
//! browser runs, over files too large to hold in memory. `count_non_null_leaves`
//! is `#[cfg(test)]`-only and exists as its oracle — `mod streaming` asserts the
//! two agree, including counting a nested object's *key* as a leaf; see
//! `a_nested_objects_key_is_counted_as_a_leaf`.
//!
//! The streaming reader takes an `upload::bytes::ChunkSource` rather than a
//! `Blob`, so `count_value` compiles and runs on the host and can be tested at
//! chunk boundaries the 16 MiB default would otherwise put out of reach.

#[cfg(any(test, target_arch = "wasm32"))]
use crate::ColumnResult;

// ── Pure, platform-agnostic JSON value counter ───────────────────────

/// Counts the non-null leaf values inside a JSON value.
///
/// Mirrors the counting semantics of the wasm-only `count_value` function but
/// operates on a complete `&str` rather than a streaming `BlobCursor`:
/// - non-empty strings → 1
/// - numbers, `true`, `false` → 1
/// - `null` → 0
/// - objects/arrays → sum of all contained values recursively
///
/// which counts every `"` occurrence as a leaf value).
///
/// Only compiled under `#[cfg(test)]` — the production wasm scanner has its
/// own `count_value` implementation; this pure function exists solely as a
/// reference for native unit testing.
#[cfg(test)]
fn count_non_null_leaves(input: &str) -> u64 {
    let (count, _) = scan_json_value(input.as_bytes(), 0);
    count
}

#[cfg(test)]
fn skip_ws(input: &[u8], mut pos: usize) -> usize {
    while input
        .get(pos)
        .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
    {
        pos += 1;
    }
    pos
}

/// Scans a JSON value from `input` starting at `pos`, returning the count of
/// non-null leaf values and the index past the value's final byte.
#[cfg(test)]
fn scan_json_value(input: &[u8], start: usize) -> (u64, usize) {
    let mut pos = skip_ws(input, start);
    let Some(&byte) = input.get(pos) else {
        return (0, pos);
    };

    match byte {
        b'"' => {
            // String — count 1 if it has at least one character (or escape).
            pos += 1; // consume opening quote
            let mut non_empty = false;
            let mut escaped = false;
            while let Some(&b) = input.get(pos) {
                if escaped {
                    escaped = false;
                    non_empty = true;
                    pos += 1;
                } else if b == b'\\' {
                    escaped = true;
                    non_empty = true;
                    pos += 1;
                } else if b == b'"' {
                    pos += 1;
                    break;
                } else {
                    non_empty = true;
                    pos += 1;
                }
            }
            (u64::from(non_empty), pos)
        }
        b'{' | b'[' => {
            let closer = if byte == b'{' { b'}' } else { b']' };
            pos += 1;
            let mut count = 0u64;

            loop {
                pos = skip_ws(input, pos);
                let Some(&b) = input.get(pos) else {
                    break; // truncated — return what we have
                };
                if b == closer {
                    pos += 1;
                    break;
                }
                if matches!(b, b',' | b':') {
                    pos += 1;
                    continue;
                }

                let (child_count, consumed) = scan_json_value(input, pos);
                count += child_count;
                // Guard against infinite loop if scan_json_value fails to
                // advance (e.g. on unexpected input that isn't a valid JSON
                // value).
                if consumed == pos {
                    pos += 1;
                } else {
                    pos = consumed;
                }
            }
            (count, pos)
        }
        _ => {
            // Bare scalar: number, true, false, or null.
            let token_start = pos;
            while input.get(pos).is_some_and(|b| {
                !matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b',' | b':' | b'}' | b']')
            }) {
                pos += 1;
            }
            if input.get(token_start..pos) == Some(b"null".as_slice()) {
                (0, pos)
            } else {
                (1, pos)
            }
        }
    }
}

// ── Wasm-only streaming scanner ──────────────────────────────────────

#[cfg(target_arch = "wasm32")]
mod wasm_entry;
#[cfg(target_arch = "wasm32")]
pub(crate) use wasm_entry::begin_scan_from_blob;

// The reader itself, not the browser: needed by the three scanners, which are
// `cfg(any(test, wasm32))` so a host test can reach the code that runs in the
// browser. `Blob` and `BlobSource` stay wasm-only because they are the browser,
// and the two functions that name them live in `wasm_entry` for the reason
// `.cargo/mutants.toml` gives: a mutant in them cannot be killed by a host test
// run, because there is no host test run that can reach them.
#[cfg(any(test, target_arch = "wasm32"))]
mod strings;

#[cfg(any(test, target_arch = "wasm32"))]
use upload::{BlobCursor, ChunkSource, UploadError};

#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) use strings::{read_json_key, skip_string_nonempty};

/// The progress line the scanner shows while it reads.
///
/// Split out of the callback that used to build it inline, and out of the
/// browser with it, for the reason the rest of this file is `cfg(any(test, …))`:
/// three arithmetic steps — the divide-by-zero guard, the clamp on both the byte
/// count and the percentage — that decide what the user reads on screen, and
/// that had no test because the function holding them took a browser `Blob`.
///
/// The guard matters for an empty upload, where `total` is zero and the
/// percentage is the only thing that would divide by it.
#[cfg(any(test, target_arch = "wasm32"))]
fn progress_message(processed: u64, total: u64) -> String {
    let safe_total = total.max(1);
    let displayed = processed.min(safe_total);
    let percent = (displayed * 100 / safe_total).min(100);
    format!("Scanning {displayed}/{safe_total} bytes ({percent}%)...")
}

/// The column grammar over any [`ChunkSource`], reporting progress.
///
/// The browser version is this with a `BlobSource` in the first argument; see
/// [`scan_columns`], which has the grammar in it.
#[cfg(any(test, target_arch = "wasm32"))]
#[allow(clippy::future_not_send)] // wasm async functions capture non-Send browser JS futures
async fn scan_source_with_progress<F, S>(
    source: S,
    total_bytes: u64,
    on_progress: F,
) -> Result<Vec<ColumnResult>, String>
where
    F: FnMut(u64, u64),
    S: ChunkSource,
{
    let mut cur = BlobCursor::new(source, total_bytes, on_progress);
    scan_columns(&mut cur).await
}

// ---------------------------------------------------------------------------
// Streaming JSON scanner - uses BlobCursor from upload crate
// ---------------------------------------------------------------------------

/// Unescapes a raw (still-escaped) JSON string body.
/// Handles standard JSON escapes including `\uXXXX` and UTF-16 surrogate pairs.
#[cfg(any(test, target_arch = "wasm32"))]
fn unescape_json_string(raw: &[u8]) -> String {
    if !raw.contains(&b'\\') {
        return String::from_utf8_lossy(raw).into_owned();
    }

    let mut out = String::with_capacity(raw.len());
    let mut i = 0;
    while let Some(&byte) = raw.get(i) {
        let entered_at = i;
        if byte == b'\\' {
            let Some(&escaped) = raw.get(i + 1) else {
                if let Some(chunk) = raw.get(i..) {
                    out.push_str(&String::from_utf8_lossy(chunk));
                }
                break;
            };
            match escaped {
                b'"' => {
                    out.push('"');
                    i += 2;
                }
                b'\\' => {
                    out.push('\\');
                    i += 2;
                }
                b'/' => {
                    out.push('/');
                    i += 2;
                }
                b'b' => {
                    out.push('\u{8}');
                    i += 2;
                }
                b'f' => {
                    out.push('\u{c}');
                    i += 2;
                }
                b'n' => {
                    out.push('\n');
                    i += 2;
                }
                b'r' => {
                    out.push('\r');
                    i += 2;
                }
                b't' => {
                    out.push('\t');
                    i += 2;
                }
                b'u' if raw.get(i + 2..i + 6).is_some() => {
                    if let Some(code) = unicode_code_unit(raw, i + 2) {
                        if (0xD800..=0xDBFF).contains(&code)
                            && raw.get(i + 6..i + 8) == Some(b"\\u")
                            && let Some(low) = unicode_code_unit(raw, i + 8)
                            && (0xDC00..=0xDFFF).contains(&low)
                        {
                            let scalar = 0x10000
                                + ((u32::from(code) - 0xD800) << 10)
                                + (u32::from(low) - 0xDC00);
                            out.push(char::from_u32(scalar).unwrap_or('\u{FFFD}'));
                            i += 12;
                        } else if (0xD800..=0xDFFF).contains(&code) {
                            // Lone high or lone low UTF-16 surrogate emits U+FFFD.
                            out.push('\u{FFFD}');
                            i += 6;
                        } else {
                            out.push(char::from_u32(u32::from(code)).unwrap_or('\u{FFFD}'));
                            i += 6;
                        }
                    } else {
                        i += 6;
                    }
                }
                other => {
                    out.push(other as char);
                    i += 2;
                }
            }
        } else {
            let Some(rest) = raw.get(i..) else {
                break;
            };
            let next = rest
                .iter()
                .position(|&c| c == b'\\')
                .map_or(raw.len(), |p| i + p);
            if let Some(chunk) = raw.get(i..next) {
                out.push_str(&String::from_utf8_lossy(chunk));
            }
            i = next;
        }

        debug_assert!(
            i > entered_at,
            "the scan must move forward: it entered at {entered_at} and is still at {i}"
        );
    }
    out
}

/// Parses four ASCII hex digits starting at `at` into a UTF-16 code unit.
#[cfg(any(test, target_arch = "wasm32"))]
fn unicode_code_unit(raw: &[u8], at: usize) -> Option<u16> {
    let digits = raw.get(at..at + 4)?;
    let hex = std::str::from_utf8(digits).ok()?;
    u16::from_str_radix(hex, 16).ok()
}

/// The Unicode scalar a `\uXXXX` escape denotes, with `XXXX` starting at `at`.
#[cfg(test)]
fn unicode_escape(raw: &[u8], at: usize) -> Option<char> {
    unicode_code_unit(raw, at).and_then(|code| char::from_u32(u32::from(code)))
}

/// How many buffer passes one scan of a value may take.
///
/// Every pass of a scan either consumes at least one byte or ends the scan, and
/// no pass can consume a byte the stream does not have, so the stream's own byte
/// count is a bound no well-formed input reaches. It is there for the pass that
/// consumes none: a reader that has stopped handing out chunks, or a loop whose
/// exit condition has been mutated away, ends after this many passes rather than
/// repeating the same pass for ever.
///
/// That is not a hypothetical. Before these loops were bounded, every timeout a
/// `cargo mutants` run reported in this file was one of them — a `+=` mutated to
/// `*=` in the cursor the loop advanced by — and a timeout is the mutation
/// report's version of a hang, which is a defect this scanner is supposed to
/// not have on a truncated upload.
///
/// One extra pass, because the pass that finds the closing quote consumes it
/// without having consumed a delimiter first.
#[cfg(any(test, target_arch = "wasm32"))]
fn pass_bound<F: FnMut(u64, u64), S: ChunkSource>(cursor: &BlobCursor<F, S>) -> usize {
    usize::try_from(cursor.total_bytes())
        .unwrap_or(usize::MAX)
        .saturating_add(1)
}

#[cfg(any(test, target_arch = "wasm32"))]
const MAX_SCALAR_BYTES: usize = 65_536;

#[cfg(any(test, target_arch = "wasm32"))]
fn push_scalar_byte(token_buf: &mut Vec<u8>, byte: u8) -> Result<(), UploadError> {
    if token_buf.len() >= MAX_SCALAR_BYTES {
        return Err(UploadError::other("JSON scalar exceeds maximum length"));
    }
    token_buf.push(byte);
    Ok(())
}

#[cfg(any(test, target_arch = "wasm32"))]
fn validate_bare_scalar(token: &[u8]) -> Result<u64, UploadError> {
    if token == b"null" {
        return Ok(0);
    }
    if token == b"true" || token == b"false" {
        return Ok(1);
    }
    if is_valid_json_number(token) {
        return Ok(1);
    }
    Err(UploadError::other("Invalid JSON literal or number"))
}

#[cfg(any(test, target_arch = "wasm32"))]
fn is_valid_json_number(bytes: &[u8]) -> bool {
    let rest = bytes.strip_prefix(b"-").unwrap_or(bytes);
    let rest = if let Some(after_zero) = rest.strip_prefix(b"0") {
        after_zero
    } else if rest.first().is_some_and(u8::is_ascii_digit) {
        let non_digit = rest
            .iter()
            .position(|b| !b.is_ascii_digit())
            .unwrap_or(rest.len());
        rest.get(non_digit..).unwrap_or_default()
    } else {
        return false;
    };
    let rest = if let Some(frac) = rest.strip_prefix(b".") {
        let frac_digits = frac
            .iter()
            .position(|b| !b.is_ascii_digit())
            .unwrap_or(frac.len());
        if frac_digits == 0 {
            return false;
        }
        frac.get(frac_digits..).unwrap_or_default()
    } else {
        rest
    };
    let rest = if let Some(exp) = rest.strip_prefix(b"e").or_else(|| rest.strip_prefix(b"E")) {
        let exp = exp
            .strip_prefix(b"+")
            .or_else(|| exp.strip_prefix(b"-"))
            .unwrap_or(exp);
        let exp_digits = exp
            .iter()
            .position(|b| !b.is_ascii_digit())
            .unwrap_or(exp.len());
        if exp_digits == 0 {
            return false;
        }
        exp.get(exp_digits..).unwrap_or_default()
    } else {
        rest
    };
    rest.is_empty()
}

/// Counts the leaves inside one `{...}` or `[...]`, the cursor sitting on its
/// opening bracket.
#[cfg(any(test, target_arch = "wasm32"))]
#[allow(clippy::future_not_send)] // wasm async functions capture non-Send browser JS futures
async fn scan_container<F, S>(cursor: &mut BlobCursor<F, S>) -> Result<u64, UploadError>
where
    F: FnMut(u64, u64),
    S: ChunkSource,
{
    let open = cursor
        .current_byte()
        .ok_or_else(|| UploadError::other("Expected '{' or '['"))?;
    let initial_close = if open == b'{' { b'}' } else { b']' };
    cursor.advance(1);
    let mut stack = vec![initial_close];
    let mut count: u64 = 0;
    let mut in_string = false;
    let mut string_has_content = false;
    let mut escaped = false;
    let mut in_token = false;
    let mut token_buf = Vec::new();

    for _ in 0..pass_bound(cursor) {
        let start = cursor.pos();
        let (consumed, closed) = {
            let Some(tail) = cursor.buffer().get(start..) else {
                break;
            };
            let mut consumed = 0usize;
            let mut closed = false;

            for (i, &b) in tail.iter().enumerate() {
                consumed = i + 1;
                if in_string {
                    if escaped {
                        escaped = false;
                        string_has_content = true;
                    } else if b == b'\\' {
                        escaped = true;
                        string_has_content = true;
                    } else if b == b'"' {
                        in_string = false;
                        if string_has_content {
                            count += 1;
                        }
                    } else {
                        string_has_content = true;
                    }
                    continue;
                }

                if in_token {
                    if matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b',' | b':' | b'}' | b']') {
                        count += validate_bare_scalar(&token_buf)?;
                        token_buf.clear();
                        in_token = false;
                    } else {
                        push_scalar_byte(&mut token_buf, b)?;
                        continue;
                    }
                }

                match b {
                    b'"' => {
                        in_string = true;
                        string_has_content = false;
                    }
                    b'{' => stack.push(b'}'),
                    b'[' => stack.push(b']'),
                    b'}' | b']' => {
                        if stack.pop() != Some(b) {
                            return Err(UploadError::other("Mismatched closing delimiter in JSON"));
                        }
                        if stack.is_empty() {
                            closed = true;
                            break;
                        }
                    }
                    b':' | b',' | b' ' | b'\t' | b'\n' | b'\r' => {}
                    _ => {
                        in_token = true;
                        token_buf.clear();
                        push_scalar_byte(&mut token_buf, b)?;
                    }
                }
            }

            (consumed, closed)
        };
        cursor.advance(consumed);

        if closed {
            return Ok(count);
        }

        if !cursor.fill().await? {
            return Err(UploadError::other(
                "Unexpected EOF while scanning nested JSON value",
            ));
        }
    }

    Err(UploadError::other(
        "Nested JSON value did not close within the document",
    ))
}

/// Counts the number of non-null "leaf" values inside a JSON value.
///
/// Nested objects and arrays are flattened and counted recursively in a single
/// pass, and a nested object's key counts like any other string in it. Strings
/// count as 1 if non-empty and 0 if empty, numbers and booleans count as 1,
/// and `null` counts as 0.
///
/// The empty-string rule is the one that was once inconsistent: this function
/// applied it via [`skip_string_nonempty`] while [`scan_container`] counted
/// every string token, so `{"":1}` was 1 here and 2 there.
///
/// Compiled for tests as well as wasm; see [`read_json_key`].
#[cfg(any(test, target_arch = "wasm32"))]
#[allow(clippy::future_not_send)] // wasm async functions capture non-Send browser JS futures
async fn count_value<F: FnMut(u64, u64), S: ChunkSource>(
    cursor: &mut BlobCursor<F, S>,
) -> Result<u64, UploadError> {
    if !cursor.ensure_any().await? {
        return Ok(0);
    }

    let first = cursor
        .current_byte()
        .ok_or_else(|| UploadError::other("Unexpected end of buffer"))?;

    if first == b'"' {
        return Ok(u64::from(skip_string_nonempty(cursor).await?));
    }

    if first == b'{' || first == b'[' {
        return scan_container(cursor).await;
    }

    // Bare scalar at this position: number, true, false, or null.
    let mut token_buf = Vec::new();
    push_scalar_byte(&mut token_buf, first)?;
    cursor.advance(1);
    for _ in 0..pass_bound(cursor) {
        let (consumed, at_delimiter) = {
            let Some(tail) = cursor.buffer().get(cursor.pos()..) else {
                break;
            };
            let mut consumed = 0usize;
            let mut at_delimiter = false;
            for &b in tail {
                if matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b',' | b':' | b'}' | b']') {
                    at_delimiter = true;
                    break;
                }
                push_scalar_byte(&mut token_buf, b)?;
                consumed += 1;
            }
            (consumed, at_delimiter)
        };

        cursor.advance(consumed);

        if at_delimiter || !cursor.fill().await? {
            return validate_bare_scalar(&token_buf);
        }
    }

    Err(UploadError::other(
        "Scalar ran past the end of the document",
    ))
}

/// Scans one JSON object (after its opening `{` has been consumed), adding
/// each key's non-null count to `fields` in first-seen key order.
///
/// When `per_row` is `false` (top-level column object), a key's value adds its
/// full leaf count; when `true` (an element of a top-level array of row
/// objects), a key's value adds `1` if non-null/non-empty and `0` otherwise.
#[cfg(any(test, target_arch = "wasm32"))]
#[allow(clippy::future_not_send)] // wasm async functions capture non-Send browser JS futures
async fn scan_object<F, S>(
    cur: &mut BlobCursor<F, S>,
    fields: &mut Vec<ColumnResult>,
    index_by_key: &mut std::collections::HashMap<String, usize>,
    per_row: bool,
) -> Result<(), String>
where
    F: FnMut(u64, u64),
    S: ChunkSource,
{
    cur.skip_ws().await.map_err(|e| e.to_string())?;
    if cur.peek().await.map_err(|e| e.to_string())? == Some(b'}') {
        cur.next_byte().await.map_err(|e| e.to_string())?;
        return Ok(());
    }

    loop {
        cur.skip_ws().await.map_err(|e| e.to_string())?;
        let key = read_json_key(cur).await.map_err(|e| e.to_string())?;
        cur.skip_ws().await.map_err(|e| e.to_string())?;

        let colon = cur.next_byte().await.map_err(|e| e.to_string())?;
        if colon != Some(b':') {
            return Err("Expected ':' after object key".to_string());
        }

        cur.skip_ws().await.map_err(|e| e.to_string())?;
        let raw_count = count_value(cur).await.map_err(|e| e.to_string())?;
        let count = if per_row {
            u64::from(raw_count > 0)
        } else {
            raw_count
        };
        if let Some(&idx) = index_by_key.get(&key) {
            if let Some(col) = fields.get_mut(idx) {
                col.count += count;
            }
        } else {
            index_by_key.insert(key.clone(), fields.len());
            fields.push(ColumnResult { key, count });
        }

        cur.skip_ws().await.map_err(|e| e.to_string())?;
        match cur.next_byte().await.map_err(|e| e.to_string())? {
            Some(b',') => {
                cur.skip_ws().await.map_err(|e| e.to_string())?;
                if cur.peek().await.map_err(|e| e.to_string())? == Some(b'}') {
                    return Err("Trailing comma in JSON object".to_string());
                }
            }
            Some(b'}') => break,
            Some(_) => return Err("Expected ',' or '}' in JSON object".to_string()),
            None => return Err("Unexpected EOF: missing closing '}'".to_string()),
        }
    }

    Ok(())
}

/// The JSON column grammar, over any [`ChunkSource`].
///
/// Accepts both a top-level column object (`{"col": [...]}`) and a top-level
/// array of row objects (`[{"col": ...}, ...]`).
#[cfg(any(test, target_arch = "wasm32"))]
#[allow(clippy::future_not_send)] // wasm async functions capture non-Send browser JS futures
async fn scan_columns<F, S>(cur: &mut BlobCursor<F, S>) -> Result<Vec<ColumnResult>, String>
where
    F: FnMut(u64, u64),
    S: ChunkSource,
{
    cur.skip_ws().await.map_err(|e| e.to_string())?;
    let Some(open) = cur.next_byte().await.map_err(|e| e.to_string())? else {
        return Ok(Vec::new());
    };

    let mut fields = Vec::new();
    let mut index_by_key = std::collections::HashMap::new();

    match open {
        b'{' => {
            scan_object(cur, &mut fields, &mut index_by_key, false).await?;
        }
        b'[' => {
            cur.skip_ws().await.map_err(|e| e.to_string())?;
            if cur.peek().await.map_err(|e| e.to_string())? == Some(b']') {
                cur.next_byte().await.map_err(|e| e.to_string())?;
            } else {
                loop {
                    cur.skip_ws().await.map_err(|e| e.to_string())?;
                    if cur.next_byte().await.map_err(|e| e.to_string())? != Some(b'{') {
                        return Err("Expected a top-level JSON object".to_string());
                    }
                    scan_object(cur, &mut fields, &mut index_by_key, true).await?;
                    cur.skip_ws().await.map_err(|e| e.to_string())?;
                    match cur.next_byte().await.map_err(|e| e.to_string())? {
                        Some(b',') => {
                            cur.skip_ws().await.map_err(|e| e.to_string())?;
                            if cur.peek().await.map_err(|e| e.to_string())? == Some(b']') {
                                return Err("Trailing comma in JSON array".to_string());
                            }
                        }
                        Some(b']') => break,
                        Some(_) => return Err("Expected ',' or ']' in JSON array".to_string()),
                        None => return Err("Unexpected EOF: missing closing ']'".to_string()),
                    }
                }
            }
        }
        _ => return Err("Expected a top-level JSON object".to_string()),
    }

    cur.skip_ws().await.map_err(|e| e.to_string())?;
    if cur.peek().await.map_err(|e| e.to_string())?.is_some() {
        return Err("Unexpected trailing content after JSON document".to_string());
    }
    Ok(fields)
}

#[cfg(test)]
mod tests;

/// Tests for the two string-escaping functions, which are host-buildable
/// precisely because they touch nothing browser-specific.
///
/// A module of their own, rather than part of `mod tests`, so that `cargo
/// mutants` attributes them to the two functions above. That is the whole reason
/// for the `cfg(any(test, …))` on those functions: before it they were compiled
/// out of the build the tests run in, and not one of the 125 mutants in this file
/// could be killed.
///
/// Being a module of its own is what does that; being a separate *file* is not.
/// These three moved to `processing/` so that this file was 1,418 lines and is
/// now 672, with `strings.rs` beside them for the two string scanners. `cargo
/// mutants --list` before and after the split: the same 451 mutants in the same
/// files, this one carrying 139 of them then and 113 in total across
/// `processing.rs` and `strings.rs` now. The other 26 are not gone to a
/// different file: most were `i += 1` on a scan cursor, which is a mutant that
/// cannot be killed except by removing it, so it was removed.
#[cfg(test)]
mod escaping;

/// Tests for the *streaming* scanner — the one that runs in the browser.
///
/// A module of its own, beside `mod tests`, `mod escaping` and `mod strings`, for
/// the same reason those are: so `cargo mutants` attributes them to the scanner
/// above.
///
/// Everything below is reachable only because the reader became source-generic.
/// Before, these functions took a browser `Blob`, so this module could not exist:
/// a test of the production JSON parser needed a browser, which is why a second
/// in-memory implementation of the same grammar was written and tested instead,
/// and the two were free to disagree.
#[cfg(test)]
mod streaming;
