//! Streaming JSON field scanner — wasm-only.
//!
//! Counts non-null values per top-level key of an uploaded JSON object using a
//! streaming `BlobCursor`, keeping memory bounded for multi-gigabyte files.

use crate::ColumnResult;
use dioxus::prelude::*;
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
