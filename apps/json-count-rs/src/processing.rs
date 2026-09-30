// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! Streaming JSON field scanner.
//!
//! Counts non-null values per top-level key of an uploaded JSON object using a
//! streaming `BlobCursor` (wasm), keeping memory bounded for multi-gigabyte
//! files.
//!
//! The platform-agnostic counting core (`count_non_null_leaves`) is separated
//! from the wasm-only streaming glue so it can be unit-tested natively.

#[cfg(target_arch = "wasm32")]
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
use upload::{Blob, BlobCursor, UploadError};

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
            // least `i` and at most `raw.len()`, so the span does too.
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
#[cfg(target_arch = "wasm32")]
#[allow(clippy::future_not_send)] // wasm async functions capture non-Send browser JS futures
async fn read_json_key<F: FnMut(u64, u64)>(
    cursor: &mut BlobCursor<F>,
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
            cursor.advance(1); // consume closing quote
            break;
        }

        if !cursor.fill().await? {
            return Err(UploadError::other("Unexpected EOF while reading string"));
        }
    }

    Ok(unescape_json_string(&raw))
}

/// Skips over a JSON string (consuming opening/closing quotes) and
/// reports only whether it had at least one character. No allocation.
#[cfg(target_arch = "wasm32")]
#[allow(clippy::future_not_send)] // wasm async functions capture non-Send browser JS futures
async fn skip_string_nonempty<F: FnMut(u64, u64)>(
    cursor: &mut BlobCursor<F>,
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

/// Counts the number of non-null "leaf" values inside a JSON value.
/// Nested objects/arrays are flattened and counted recursively in a
/// single synchronous pass; strings count as 1 if non-empty; numbers
/// and booleans count as 1; `null` counts as 0.
#[cfg(target_arch = "wasm32")]
#[allow(clippy::future_not_send)] // wasm async functions capture non-Send browser JS futures
async fn count_value<F: FnMut(u64, u64)>(cursor: &mut BlobCursor<F>) -> Result<u64, UploadError> {
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
        cursor.advance(1);
        let mut depth: i32 = 1;
        let mut count: u64 = 0;
        let mut in_string = false;
        let mut escaped = false;
        let mut in_token = false;
        let mut token_first_byte = 0u8;

        loop {
            let buf = cursor.buffer().to_vec(); // Copy to avoid borrow issues
            let mut i = cursor.pos();

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
                            let advance_by = i - cursor.pos();
                            cursor.advance(advance_by);
                            return Ok(count);
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

            let advance_by = i - cursor.pos();
            cursor.advance(advance_by);

            if !cursor.fill().await? {
                return Err(UploadError::other(
                    "Unexpected EOF while scanning nested JSON value",
                ));
            }
        }
    }

    // Bare scalar at this position: number, true, false, or null.
    let first = cursor.current_byte().unwrap_or(b'n');
    cursor.advance(1);
    loop {
        let buf = cursor.buffer().to_vec(); // Copy to avoid borrow issues
        let i = cursor.pos();
        while let Some(&b) = buf.get(i) {
            if matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b',' | b':' | b'}' | b']') {
                return Ok(u64::from(first != b'n'));
            }
            cursor.advance(1);
        }
        if !cursor.fill().await? {
            return Ok(u64::from(first != b'n'));
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::future_not_send)] // wasm async functions capture non-Send browser JS futures
async fn scan_blob_with_progress(
    blob: &Blob,
    on_progress: impl FnMut(u64, u64),
) -> Result<Vec<ColumnResult>, String> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    // browser Blob.size() returns f64, cast to u64 for byte counts
    let total_bytes = blob.size() as u64;
    let mut cur = BlobCursor::new(blob, total_bytes, on_progress);

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

        let key = read_json_key(&mut cur).await.map_err(|e| e.to_string())?;
        cur.skip_ws().await.map_err(|e| e.to_string())?;

        let colon = cur.next_byte().await.map_err(|e| e.to_string())?;
        if colon != Some(b':') {
            return Err("Expected ':' after object key".to_string());
        }

        cur.skip_ws().await.map_err(|e| e.to_string())?;
        let count = count_value(&mut cur).await.map_err(|e| e.to_string())?;
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
