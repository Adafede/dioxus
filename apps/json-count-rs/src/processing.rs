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
//! two agree, which is what makes the one difference between them (the streaming
//! side counts a nested object's *key* as a leaf) visible at all. That
//! divergence is pinned, not endorsed; see
//! `a_nested_objects_key_is_counted_as_a_leaf_which_may_not_be_intended`.
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
/// single-pass tokeniser with five pieces of state to carry across chunk
/// boundaries, and folding it into its caller put it over the line limit where
/// neither could be read on its own.
///
/// It counts a nested object's key as well as its value, which is why
/// `{"y":3}` is 2 and not 1. `count_value` counts the same way, and the
/// test named for it pins that.
///
/// A string counts only if it has content, matching [`count_value`] and the
/// doc on that function: `""` is 0. The count is therefore deferred to the
/// closing quote rather than added at the opening one, which is what
/// `string_has_content` tracks. It used to be added at the opening quote
/// unconditionally, so `{"":1}` was 2 here and 1 through `count_value` —
/// the one place the two implementations of this grammar disagreed.
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
    let mut string_has_content = false;
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
                    // Mirrors `skip_string_nonempty`: a backslash and the byte
                    // it escapes both count as content, so `"\""` is 1.
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
                        string_has_content = false;
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
/// These three moved to `processing/` so that this file is 791 lines instead of
/// 1418. `cargo mutants --list` was compared before and after: the same 451
/// mutants in the same files, this one still carrying 139 of them.
#[cfg(test)]
mod escaping;

/// Tests for the *streaming* scanner — the one that runs in the browser.
///
/// A module of its own, beside `mod tests` and `mod escaping`, for the same
/// reason those are: so `cargo mutants` attributes them to the scanner above.
///
/// Everything below is reachable only because the reader became source-generic.
/// Before, these functions took a browser `Blob`, so this module could not exist:
/// a test of the production JSON parser needed a browser, which is why a second
/// in-memory implementation of the same grammar was written and tested instead,
/// and the two were free to disagree.
#[cfg(test)]
mod streaming;
