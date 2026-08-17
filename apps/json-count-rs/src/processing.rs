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
    while pos < input.len() && matches!(input[pos], b' ' | b'\t' | b'\n' | b'\r') {
        pos += 1;
    }
    pos
}

/// Scans a JSON value from `input` starting at `pos`, returning the count of
/// non-null leaf values and the index past the value's final byte.
#[cfg(test)]
fn scan_json_value(input: &[u8], start: usize) -> (u64, usize) {
    let mut pos = skip_ws(input, start);
    if pos >= input.len() {
        return (0, pos);
    }

    match input[pos] {
        b'"' => {
            // String — count 1 if it has at least one character (or escape).
            pos += 1; // consume opening quote
            let mut non_empty = false;
            let mut escaped = false;
            while pos < input.len() {
                let b = input[pos];
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
            let opener = input[pos];
            let closer = if opener == b'{' { b'}' } else { b']' };
            pos += 1;
            let mut count = 0u64;

            loop {
                pos = skip_ws(input, pos);
                if pos >= input.len() {
                    break; // truncated — return what we have
                }
                if input[pos] == closer {
                    pos += 1;
                    break;
                }
                if input[pos] == b',' || input[pos] == b':' {
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
            while pos < input.len()
                && !matches!(
                    input[pos],
                    b' ' | b'\t' | b'\n' | b'\r' | b',' | b':' | b'}' | b']'
                )
            {
                pos += 1;
            }
            let token = &input[token_start..pos];
            if token == b"null" { (0, pos) } else { (1, pos) }
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
#[cfg(target_arch = "wasm32")]
fn unescape_json_string(raw: &[u8]) -> String {
    if !raw.contains(&b'\\') {
        return String::from_utf8_lossy(raw).into_owned();
    }

    let mut out = String::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'\\' && i + 1 < raw.len() {
            match raw[i + 1] {
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
                b'u' if i + 6 <= raw.len() => {
                    if let Ok(hex) = std::str::from_utf8(&raw[i + 2..i + 6])
                        && let Ok(code) = u32::from_str_radix(hex, 16)
                        && let Some(c) = char::from_u32(code)
                    {
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
            let next = raw[i..]
                .iter()
                .position(|&c| c == b'\\')
                .map_or(raw.len(), |p| i + p);
            out.push_str(&String::from_utf8_lossy(&raw[i..next]));
            i = next;
        }
    }
    out
}

/// Reads a JSON string key from the cursor using the shared BlobCursor.
#[cfg(target_arch = "wasm32")]
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

        while i < buf.len() {
            match buf[i] {
                _ if escaped => {
                    escaped = false;
                    i += 1;
                    continue;
                }
                b'\\' => escaped = true,
                b'"' => {
                    closed = true;
                    break;
                }
                _ => {
                    i += 1;
                }
            }
        }

        raw.extend_from_slice(&buf[start..i]);
        cursor.advance(i - start);

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

        while i < buf.len() {
            match buf[i] {
                _ if escaped => {
                    escaped = false;
                    i += 1;
                    continue;
                }
                b'\\' => escaped = true,
                b'"' => {
                    closed = true;
                    break;
                }
                _ => {
                    i += 1;
                }
            }
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

            while i < buf.len() {
                let b = buf[i];

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
        while i < buf.len() {
            let b = buf[i];
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
#[allow(clippy::future_not_send)]
async fn scan_blob_with_progress(
    blob: &Blob,
    on_progress: impl FnMut(u64, u64),
) -> Result<Vec<ColumnResult>, String> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
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
