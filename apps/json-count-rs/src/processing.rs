// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! Streaming JSON field scanner.
//!
//! Counts non-null values per top-level key of an uploaded JSON object using a
//! streaming `BlobCursor`, keeping memory bounded for multi-gigabyte files.
//!
//! # Two implementations of one grammar, and why that was a mistake
//!
//! This file used to hold two: `count_non_null_leaves`, an in-memory scanner over
//! a `&str`, and the streaming one above it, which is the one that actually runs
//! in the browser over files too large to hold in memory. The first was tested and
//! the second was not, and the reason was not a decision — it was that the second
//! took a browser `Blob`, once through `BlobCursor`, so `cfg(target_arch =
//! "wasm32")` was the only honest gate and every test of it would have needed a
//! browser.
//!
//! `upload::bytes::ChunkSource` removes that reason. The streaming reader now
//! takes a source rather than a `Blob`, so the streaming scanner compiles and runs
//! on the host, and `mod streaming` tests it — including at chunk boundaries,
//! which the 16 MiB default put permanently out of reach.
//!
//! `count_non_null_leaves` is still here and still tested. Whether one grammar
//! should be two implementations is a real question and this change is not the
//! answer to it; what it removes is the excuse for not knowing which one the
//! browser was using.

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
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use upload::{Blob, BlobSource};

// The reader itself, not the browser: needed by the three scanners, which are
// `cfg(any(test, wasm32))` so a host test can reach the code that runs in the
// browser. `Blob` and `BlobSource` stay wasm-only because they are the browser.
#[cfg(any(test, target_arch = "wasm32"))]
use upload::{BlobCursor, ChunkSource, UploadError};

#[cfg(target_arch = "wasm32")]
pub(crate) fn begin_scan_from_blob(
    blob: Blob,
    file_name: String,
    mut file_name_signal: Signal<String>,
    mut status: Signal<String>,
    mut results: Signal<Vec<ColumnResult>>,
    mut busy: Signal<bool>,
    mut drag_active: Signal<bool>,
) {
    file_name_signal.set(file_name);
    busy.set(true);
    drag_active.set(false);
    status.set("Reading file...".to_string());
    results.set(vec![]);
    spawn_scan(blob, status, results, busy);
}

#[cfg(target_arch = "wasm32")]
fn spawn_scan(
    blob: Blob,
    mut status: Signal<String>,
    mut results: Signal<Vec<ColumnResult>>,
    mut busy: Signal<bool>,
) {
    spawn(async move {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // browser Blob.size() returns f64, cast to u64 for byte counts
        let total_bytes = blob.size() as u64;
        status.set(format!("Scanning {total_bytes} bytes..."));

        let cols = match scan_blob_with_progress(&blob, move |processed, total| {
            let safe_total = total.max(1);
            let displayed = processed.min(safe_total);
            let percent = (displayed * 100 / safe_total).min(100);
            status.set(format!(
                "Scanning {displayed}/{safe_total} bytes ({percent}%)..."
            ));
        })
        .await
        {
            Ok(cols) => cols,
            Err(error) => {
                status.set(format!("Error reading file: {error:?}"));
                Vec::new()
            }
        };

        let total: u64 = cols.iter().map(|col| col.count).sum();
        status.set(format!(
            "Done — {} columns, {} total non-null values",
            cols.len(),
            total
        ));
        results.set(cols);
        busy.set(false);
    });
}
// ---------------------------------------------------------------------------
// Streaming JSON scanner - uses BlobCursor from upload crate
// ---------------------------------------------------------------------------

/// Unescapes a raw (still-escaped) JSON string body.
/// Handles standard JSON escapes including `\uXXXX` (BMP).
///
/// Not `cfg`-gated to wasm, though it is only called from the streaming scanner
/// that is. It takes bytes and returns a `String`, so it runs anywhere — and
/// gating it meant `cargo mutants` could not reach a single one of its 125
/// mutants from a host test run, because the whole of it was compiled out of
/// the build the tests run in. The scanner's *tests* are wasm-only; this
/// function's do not need to be.
///
/// `any(test, …)` rather than `target_arch` alone, because the only caller *is*
/// the wasm scanner: without the test arm these two are dead code on the host,
/// and `dead_code` is denied.
#[cfg(any(test, target_arch = "wasm32"))]
fn unescape_json_string(raw: &[u8]) -> String {
    if !raw.contains(&b'\\') {
        return String::from_utf8_lossy(raw).into_owned();
    }

    let mut out = String::with_capacity(raw.len());
    let mut i = 0;
    while let Some(&byte) = raw.get(i) {
        // `out` is grown inside this loop, so a branch that failed to move the
        // cursor would not merely hang: it would allocate a `String` until the
        // tab died. The trailing-backslash case below is exactly that bug, and
        // it was live for the life of the function. Two invariants make the
        // class checkable instead, both of which hold for every input and
        // neither of which needs the loop to terminate to check:
        //
        // * `i` strictly increases, so the loop runs at most `raw.len()` times;
        // * `out` is never longer than `raw`, because every escape consumes at
        //   least two input bytes and emits at most one output byte, and every
        //   run of literal characters is copied at its own length.
        //
        // Both are `debug_assert!` so they cost nothing in a release build and
        // are checked in every test, which is where a regression in this
        // function would be introduced.
        let entered_at = i;
        let is_escape = byte == b'\\';
        if is_escape {
            // A backslash with nothing after it has no escape to expand, so it is
            // literal text — and the scan has to leave it behind before
            // continuing. The run below finds the *first* backslash in the tail
            // and sets `next` to its offset, so for a trailing backslash `next`
            // is `i` again and the loop would never terminate. This is the
            // malformed-input case: the string body is whatever sat between two
            // quotes in the file, and a file that ends one mid-escape produces
            // exactly this.
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
                // A `\u` with fewer than four digits left is not a unicode
                // escape at all, and falls through to the unknown-escape arm
                // below rather than swallowing six bytes.
                b'u' if raw.get(i + 2..i + 6).is_some() => {
                    if let Some(c) = unicode_escape(raw, i + 2) {
                        out.push(c);
                    }
                    i += 6;
                }
                other => {
                    out.push(other as char);
                    i += 2;
                }
            }
        } else {
            // `raw.get(i)` succeeded, so the tail from `i` exists; `next` is at
            // least `i + 1` — this branch is only reached when `raw[i]` is not a
            // backslash, so the first backslash in the tail is not at its offset
            // zero — and at most `raw.len()`, so the span does too.
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

        // After both branches, because both are what move the cursor: checked
        // earlier, the branch that does the advancing had not run yet.
        debug_assert!(
            i > entered_at,
            "the scan must move forward: it entered at {entered_at} and is still \
             at {i}, which would grow `out` without consuming input"
        );
        debug_assert!(
            out.len() <= raw.len(),
            "unescaping cannot lengthen a string: {} bytes out of {} in",
            out.len(),
            raw.len()
        );
    }
    out
}

/// The character a `\uXXXX` escape denotes, with `XXXX` starting at `at`.
///
/// `None` for any of the four ways this can fail — the digits are not there, are
/// not ASCII, are not hex, or are a surrogate rather than a code point — because
/// the caller's response is the same in every case: emit nothing and let the
/// raw bytes stand. Written as one function rather than a chain of `if let`s so
/// the four checks are each a line that can be read, and mutated, on its own.
/// Host-buildable for the same reason as `unescape_json_string` above.
#[cfg(any(test, target_arch = "wasm32"))]
fn unicode_escape(raw: &[u8], at: usize) -> Option<char> {
    let digits = raw.get(at..at + 4)?;
    let hex = std::str::from_utf8(digits).ok()?;
    let code = u32::from_str_radix(hex, 16).ok()?;
    char::from_u32(code)
}

/// Reads a JSON string key from the cursor using the shared `BlobCursor`.
///
/// Compiled for tests as well as wasm. It scans bytes and nothing else — no
/// browser API, no `Blob` — and until the reader became source-generic there was
/// no way to say so: it was `cfg(target_arch = "wasm32")` because its *parameter*
/// was, and one wasm parameter took the whole function down with it. That is why
/// this grammar had a second, in-memory implementation next to it: this one, the
/// one that runs in the browser, had no reachable test at all.
#[cfg(any(test, target_arch = "wasm32"))]
#[allow(clippy::future_not_send)] // wasm async functions capture non-Send browser JS futures
async fn read_json_key<F: FnMut(u64, u64), S: ChunkSource>(
    cursor: &mut BlobCursor<F, S>,
) -> Result<String, UploadError> {
    if cursor.next_byte().await? != Some(b'"') {
        return Err(UploadError::other("Expected opening quote for string"));
    }

    let mut raw = Vec::new();
    let mut escaped = false;
    loop {
        let start = cursor.pos();
        let buf = cursor.buffer();
        let mut i = start;
        let mut closed = false;

        while let Some(&byte) = buf.get(i) {
            match byte {
                _ if escaped => {
                    escaped = false;
                }
                b'\\' => escaped = true,
                b'"' => {
                    closed = true;
                    break;
                }
                _ if !escaped => {
                    raw.push(byte);
                }
                _ => {}
            }
            i += 1;
        }
        if closed {
            // The key body *and* its closing quote. Advancing only 1 — which is
            // what this did — leaves the cursor sitting on the closing quote, so
            // the caller's colon check reads `"` and every non-empty key fails
            // with "Expected ':' after object key". It works for `{"":1}`, where
            // the body is empty and the two advances coincide, which is why
            // nothing noticed: no test could reach this function at all.
            cursor.advance(i - start + 1);
            break;
        }

        // Consume what was scanned before refilling. Without this the buffer is
        // never drained — `fill` drains `0..pos`, and `pos` has not moved — so the
        // next pass re-scans the same bytes and appends them to `raw` again: a key
        // split across a chunk boundary came back as `nn` for `"n"` and
        // `sststrstr` for `"str"`, and a key long enough to span many chunks grew
        // `raw` without bound. `skip_string_nonempty` has always had this line.
        cursor.advance(i - start);

        if !cursor.fill().await? {
            return Err(UploadError::other("Unexpected EOF while reading string"));
        }
    }

    Ok(unescape_json_string(&raw))
}

/// Skips over a JSON string (consuming opening/closing quotes) and
/// reports only whether it had at least one character. No allocation.
///
/// Compiled for tests as well as wasm; see [`read_json_key`]. The `b'\\'` arm
/// below is the one that makes a body ending in a lone backslash scan forever,
/// and it is the arm mutants delete most often, so having a test that reaches it
/// is the point.
#[cfg(any(test, target_arch = "wasm32"))]
#[allow(clippy::future_not_send)] // wasm async functions capture non-Send browser JS futures
async fn skip_string_nonempty<F: FnMut(u64, u64), S: ChunkSource>(
    cursor: &mut BlobCursor<F, S>,
) -> Result<bool, UploadError> {
    if cursor.next_byte().await? != Some(b'"') {
        return Err(UploadError::other("Expected opening quote for string"));
    }

    let mut escaped = false;
    let mut any = false;
    loop {
        let start = cursor.pos();
        let buf = cursor.buffer();
        let mut i = start;
        let mut closed = false;

        while let Some(&byte) = buf.get(i) {
            match byte {
                _ if escaped => {
                    escaped = false;
                }
                b'\\' => escaped = true,
                b'"' => {
                    closed = true;
                    break;
                }
                _ => {}
            }
            i += 1;
        }

        if i > start {
            any = true;
        }
        cursor.advance(i - start);

        if closed {
            cursor.advance(1); // consume closing quote
            break;
        }

        if !cursor.fill().await? {
            return Err(UploadError::other("Unexpected EOF while reading string"));
        }
    }

    Ok(any)
}

/// Counts the leaves inside one `{...}` or `[...]`, the cursor sitting on its
/// opening bracket.
///
/// Split out of [`count_value`] because the two are different jobs: this one is a
/// single-pass tokeniser with four pieces of state to carry across chunk
/// boundaries, and folding it into its caller put it over the line limit where
/// neither could be read on its own.
///
/// It counts tokens, not values, which is why a nested object's key is counted
/// — see the test named for that.
#[cfg(any(test, target_arch = "wasm32"))]
#[allow(clippy::future_not_send)] // wasm async functions capture non-Send browser JS futures
async fn scan_container<F, S>(cursor: &mut BlobCursor<F, S>) -> Result<u64, UploadError>
where
    F: FnMut(u64, u64),
    S: ChunkSource,
{
    cursor.advance(1);
    let mut depth: i32 = 1;
    let mut count: u64 = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut in_token = false;
    let mut token_first_byte = 0u8;

    loop {
        let start = cursor.pos();
        // Scanned through a borrow of the buffer rather than a clone of it.
        // The clone was there to dodge a borrow conflict with the `advance`
        // calls below, and it cost a copy of the whole chunk — up to 16 MiB,
        // transiently doubling the reader's peak — once per chunk, for a
        // scanner whose entire reason to exist is bounded memory. Scoping
        // the borrow and applying the advance afterwards removes both the
        // copy and the conflict.
        let (scanned_to, closed) = {
            let buf = cursor.buffer();
            let mut i = start;
            let mut closed = false;

            while let Some(&b) = buf.get(i) {
                if in_string {
                    if escaped {
                        escaped = false;
                    } else if b == b'\\' {
                        escaped = true;
                    } else if b == b'"' {
                        in_string = false;
                    }
                    i += 1;
                    continue;
                }

                if in_token {
                    if matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b',' | b':' | b'}' | b']') {
                        if token_first_byte != b'n' {
                            count += 1;
                        }
                        in_token = false;
                    } else {
                        i += 1;
                        continue;
                    }
                }

                match b {
                    b'"' => {
                        in_string = true;
                        count += 1;
                        i += 1;
                    }
                    b'{' | b'[' => {
                        depth += 1;
                        i += 1;
                    }
                    b'}' | b']' => {
                        depth -= 1;
                        i += 1;
                        if depth == 0 {
                            closed = true;
                            break;
                        }
                    }
                    b':' | b',' | b' ' | b'\t' | b'\n' | b'\r' => {
                        i += 1;
                    }
                    _ => {
                        in_token = true;
                        token_first_byte = b;
                        i += 1;
                    }
                }
            }

            (i, closed)
        };
        cursor.advance(scanned_to - start);

        if closed {
            return Ok(count);
        }

        if !cursor.fill().await? {
            return Err(UploadError::other(
                "Unexpected EOF while scanning nested JSON value",
            ));
        }
    }
}

/// Counts the number of non-null "leaf" values inside a JSON value.
/// Nested objects/arrays are flattened and counted recursively in a
/// single synchronous pass; strings count as 1 if non-empty; numbers
/// and booleans count as 1; `null` counts as 0.
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
    let first = cursor.current_byte().unwrap_or(b'n');
    cursor.advance(1);
    loop {
        // `i` tracked the cursor but was never itself advanced, while `buf` was a
        // snapshot: the loop read the same byte forever and advanced the cursor
        // past the end of the stream. A scalar with no delimiter in the rest of
        // its chunk — which is every scalar at a chunk boundary — spun here
        // forever. On a multi-gigabyte upload that is a frozen tab, and no test
        // could reach it. Scanned by index instead, and the borrow scoped so the
        // `advance` does not need a clone of the chunk to escape it.
        let (consumed, at_delimiter) = {
            let buf = cursor.buffer();
            let mut i = cursor.pos();
            let mut consumed = 0;
            let mut at_delimiter = false;
            while let Some(&b) = buf.get(i) {
                if matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b',' | b':' | b'}' | b']') {
                    at_delimiter = true;
                    break;
                }
                i += 1;
                consumed += 1;
            }
            (consumed, at_delimiter)
        };

        cursor.advance(consumed);

        if at_delimiter || !cursor.fill().await? {
            return Ok(u64::from(first != b'n'));
        }
    }
}

/// The JSON column grammar, over any [`ChunkSource`].
///
/// This is the scanner that runs in the browser, with the browser taken out.
/// `scan_blob_with_progress` is now three lines that open a `Blob` and hand the
/// cursor over.
///
/// Extracted rather than tested through the wasm entry point, because that entry
/// point takes a browser `Blob` and so can only run in a browser. Everything that
/// made this grammar hard to test was downstream of that one parameter.
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
    if open != b'{' {
        return Err("Expected a top-level JSON object".to_string());
    }

    let mut fields = Vec::new();
    loop {
        cur.skip_ws().await.map_err(|e| e.to_string())?;
        if cur.peek().await.map_err(|e| e.to_string())? == Some(b'}') {
            cur.next_byte().await.map_err(|e| e.to_string())?;
            break;
        }

        let key = read_json_key(cur).await.map_err(|e| e.to_string())?;
        cur.skip_ws().await.map_err(|e| e.to_string())?;

        let colon = cur.next_byte().await.map_err(|e| e.to_string())?;
        if colon != Some(b':') {
            return Err("Expected ':' after object key".to_string());
        }

        cur.skip_ws().await.map_err(|e| e.to_string())?;
        let count = count_value(cur).await.map_err(|e| e.to_string())?;
        fields.push(ColumnResult { key, count });

        cur.skip_ws().await.map_err(|e| e.to_string())?;
        match cur.peek().await.map_err(|e| e.to_string())? {
            Some(b',') => {
                cur.next_byte().await.map_err(|e| e.to_string())?;
            }
            Some(b'}') => {
                cur.next_byte().await.map_err(|e| e.to_string())?;
                break;
            }
            _ => break,
        }
    }

    Ok(fields)
}

/// Opens a browser `Blob` and scans it. See [`scan_columns`], which is the part
/// with the grammar in it.
#[cfg(target_arch = "wasm32")]
#[allow(clippy::future_not_send)] // wasm async functions capture non-Send browser JS futures
async fn scan_blob_with_progress(
    blob: &Blob,
    on_progress: impl FnMut(u64, u64),
) -> Result<Vec<ColumnResult>, String> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    // browser Blob.size() returns f64, cast to u64 for byte counts
    let total_bytes = blob.size() as u64;
    let mut cur = BlobCursor::new(BlobSource::new(blob), total_bytes, on_progress);
    scan_columns(&mut cur).await
}

#[cfg(test)]
mod tests {
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
}

/// Tests for the two string-escaping functions, which are host-buildable
/// precisely because they touch nothing browser-specific.
///
/// They live here rather than inside `mod tests` so that `cargo mutants` — which
/// discovers tests by path — sees them as the tests of the functions above. That
/// is the whole reason for the `cfg(any(test, …))` on those two: before it, the
/// functions were compiled out of the build the tests run in, and not one of the
/// 125 mutants in this file could be killed.
#[cfg(test)]
mod escaping {
    use super::{unescape_json_string, unicode_escape};

    fn unescape(s: &str) -> String {
        unescape_json_string(s.as_bytes())
    }

    // ── the plain pass-through ───────────────────────────────────────────────

    #[test]
    fn a_string_with_no_backslash_is_returned_unchanged() {
        // The early return: without a `\\` there is nothing to unescape, and
        // scanning for one costs a pass over the bytes.
        assert_eq!(unescape("plain text"), "plain text", "no escapes to expand");
        assert_eq!(unescape(""), "", "and nothing at all");
    }

    #[test]
    fn a_backslash_is_what_arms_the_scanner() {
        // One backslash anywhere means the string is escaped, so the rest of the
        // function has to survive the mixed case: escapes and literals together.
        // `z` is not one of the eight escapes, so it stands for an unknown one.
        //
        // The backslash is *dropped* and only the character kept. That is lossy:
        // `a\zb` and `azb` both unescape to `azb`, so a malformed file cannot be
        // told from a well-formed one after this. Keeping the backslash would be
        // the other defensible choice, but that is a behaviour change and not
        // one to make while adding tests — so it is pinned here as what the code
        // does, which is the first step to changing it deliberately.
        assert_eq!(
            unescape(r"a\zc"),
            "azc",
            "an unknown escape contributes only its character"
        );
        assert_eq!(
            unescape(r"\za\nb"),
            "za\nb",
            "and the scan carries on from just after it"
        );
    }

    // ── the two-character escapes ───────────────────────────────────────────

    #[test]
    fn every_single_character_escape_is_expanded() {
        assert_eq!(unescape(r#"\""#), "\"", r#"\" → \""#);
        assert_eq!(unescape(r"\\"), r"\", r"\\ → \");
        assert_eq!(unescape(r"\/"), "/", r"\/ → /");
        assert_eq!(unescape(r"\b"), "\u{8}", r"\b → backspace");
        assert_eq!(unescape(r"\f"), "\u{c}", r"\f → form feed");
        assert_eq!(unescape(r"\n"), "\n", r"\n → newline");
        assert_eq!(unescape(r"\r"), "\r", r"\r → carriage return");
        assert_eq!(unescape(r"\t"), "\t", r"\t → tab");
    }

    #[test]
    fn escapes_advance_past_both_characters() {
        // Two characters per escape: advancing one would leave the second to be
        // emitted as a literal, and this is the assertion that catches it.
        assert_eq!(unescape(r"\n\n"), "\n\n", "two escapes, four bytes in");
        assert_eq!(unescape(r"a\nb"), "a\nb", "literals either side survive");
    }

    // ── \uXXXX ───────────────────────────────────────────────────────────────

    #[test]
    fn a_unicode_escape_is_the_character_it_names() {
        // 0041 is 'A'; 00e9 is 'é'. Both are BMP, which is all this handles.
        assert_eq!(unescape(r"A"), "A", r"A is A");
        assert_eq!(unescape(r"é"), "é", "00e9 is e-acute");
    }

    #[test]
    fn the_four_digits_are_read_from_the_right_place() {
        // `at` is the offset of the first digit, so reading from `at` rather
        // than `at - 1` would pick up the `u` and fail to parse as hex.
        assert_eq!(
            unicode_escape(b"\\u0041", 2),
            Some('A'),
            "the digits start after the backslash and the u"
        );
        assert_eq!(
            unicode_escape(b"0041", 0),
            Some('A'),
            "and reading from the start of a bare digit run works too"
        );
    }

    #[test]
    fn a_unicode_escape_fails_when_the_digits_are_not_there() {
        // Fewer than four bytes left: `get` returns None rather than slicing
        // past the end, and the caller emits nothing.
        assert_eq!(unicode_escape(b"", 0), None, "no bytes at all");
        assert_eq!(unicode_escape(b"00", 0), None, "two digits");
        assert_eq!(unicode_escape(b"004", 0), None, "three digits");
    }

    #[test]
    fn a_unicode_escape_fails_when_the_digits_are_not_hex() {
        assert_eq!(unicode_escape(b"00zz", 0), None, "z is not a hex digit");
        assert_eq!(unicode_escape(b"  41", 0), None, "a space is not either");
    }

    #[test]
    fn a_surrogate_is_not_a_character() {
        // D800 is a surrogate half. `char::from_u32` refuses it, and the escape
        // is dropped rather than emitting an unpaired surrogate, which would
        // produce a string Rust cannot represent.
        assert_eq!(
            unicode_escape(b"d800", 0),
            None,
            "a surrogate half is not a character"
        );
        assert!(char::from_u32(0xD800).is_none(), "which is the reason");
    }

    #[test]
    fn a_truncated_unicode_escape_falls_through_to_the_literal_branch() {
        // The `b'u'` arm is guarded on four digits being present. Without the
        // guard, a truncated escape would consume six bytes and emit nothing;
        // with it, the `u` is treated as an unknown escape and kept.
        assert_eq!(
            unescape(r"\u00"),
            "u00",
            "the u is kept and the scan continues from the digits"
        );
    }

    #[test]
    fn an_invalid_unicode_escape_emits_nothing_and_still_advances() {
        // Four digits are present but are not a character: `i += 6` happens
        // either way, so a bad escape cannot make the scan loop forever.
        assert_eq!(
            unescape(r"\ud800"),
            "",
            "an unpaired surrogate contributes no character"
        );
        assert_eq!(
            unescape(r"\ud800tail"),
            "tail",
            "and the scan carries on from after the escape"
        );
    }

    // ── the literal run between escapes ─────────────────────────────────────

    #[test]
    fn a_run_of_plain_characters_is_copied_in_one_go() {
        // The `else` branch finds the next backslash and copies everything up to
        // it. A run with no backslash at all is the early return; this is the
        // run that stops at one.
        assert_eq!(
            unescape(r"hello\nworld"),
            "hello\nworld",
            "one run, one escape"
        );
        assert_eq!(unescape(r"\na\rb\tc"), "\na\rb\tc", "a leading escape");
        assert_eq!(
            unescape(r"a\tb\tc"),
            "a\tb\tc",
            "no run before the first escape"
        );
    }

    #[test]
    fn a_run_stops_at_the_first_backslash() {
        // `position` finds the *first* backslash, so a run cannot swallow the
        // start of the next escape.
        assert_eq!(unescape(r"abc\n"), "abc\n", "and nothing past it");
    }

    #[test]
    fn a_backslash_at_the_end_is_a_literal() {
        // There is no second byte to pair with, so the condition fails and the
        // backslash is copied as ordinary text rather than panicking on
        // `raw[i + 1]`.
        assert_eq!(
            unescape("abc\\"),
            "abc\\",
            "a trailing backslash is literal"
        );
        assert_eq!(unescape("\\"), "\\", "and a lone one");
    }

    #[test]
    fn invalid_utf8_is_replaced_rather_than_rejected() {
        // The body comes from a JSON file, so it can hold bytes that are not
        // UTF-8. `from_utf8_lossy` is what keeps one bad byte from failing the
        // whole column; the replacement character is the documented cost.
        let out = unescape_json_string(&[b'a', 0xFF, b'b']);
        assert_eq!(
            out.chars().count(),
            3,
            "the bad byte becomes one replacement character"
        );
        assert!(
            out.contains('\u{FFFD}'),
            "and it is the replacement: {out:?}"
        );
    }
}

/// Tests for the *streaming* scanner — the one that runs in the browser.
///
/// They live here, beside `mod tests` and `mod escaping`, for the same reason
/// those do: `cargo mutants` discovers tests by path, so a test in the same file
/// as the function it covers is what lets a mutant be attributed to it.
///
/// Everything below is reachable only because the reader became source-generic.
/// Before, these functions took a browser `Blob`, so this module could not exist:
/// a test of the production JSON parser needed a browser, which is why a second
/// in-memory implementation of the same grammar was written and tested instead,
/// and the two were free to disagree.
#[cfg(test)]
mod streaming {
    use futures_executor::block_on;
    use upload::{BlobCursor, SliceSource};

    use super::scan_columns;

    /// A cursor over `bytes` with a deliberately tiny chunk size.
    fn cursor(bytes: &[u8], chunk: usize) -> BlobCursor<fn(u64, u64), SliceSource<'_>> {
        let on_progress: fn(u64, u64) = |_, _| {};
        BlobCursor::new(SliceSource::new(bytes), bytes.len() as u64, on_progress)
            .with_chunk_size(chunk)
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

    /// Pinned as it behaves, not as it should: the streaming scanner counts a
    /// nested object's *key* as a leaf, so `{"y":3}` contributes 2 rather than the
    /// 1 that `count_non_null_leaves` gives it. The two implementations of this
    /// grammar disagree, and this is the difference.
    ///
    /// Not decided here: it is a question about what the app should report, not
    /// about the reader — and there was no working result to preserve before the
    /// `read_json_key` fix, so nothing depended on the old number. The name says
    /// what it is so nobody reads it as intent.
    #[test]
    fn a_nested_objects_key_is_counted_as_a_leaf_which_may_not_be_intended() {
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
}
