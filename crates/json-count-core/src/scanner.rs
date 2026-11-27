// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! Streaming top-level JSON object/array column scanner.

use std::collections::HashMap;

use stream_io::{BlobCursor, ChunkSource};

use crate::container::{ScanBuffers, scan_bare_scalar, scan_container};
use crate::error::JsonCountError;
use crate::model::ColumnResult;
use crate::strings::{read_json_key_into, skip_string_nonempty};

/// Counts non-null leaf values inside any JSON value starting at `cursor`.
///
/// # Errors
///
/// Returns [`JsonCountError`] on malformed RFC 8259 JSON, invalid escapes/UTF-8, or stream failures.
#[cfg(test)]
pub(crate) async fn count_value<F: FnMut(u64, u64), S: ChunkSource>(
    cursor: &mut BlobCursor<F, S>,
) -> Result<u64, JsonCountError> {
    let mut bufs = ScanBuffers::default();
    count_value_with_bufs(cursor, &mut bufs).await
}

async fn count_value_with_bufs<F: FnMut(u64, u64), S: ChunkSource>(
    cursor: &mut BlobCursor<F, S>,
    bufs: &mut ScanBuffers,
) -> Result<u64, JsonCountError> {
    if !cursor.ensure_any().await? {
        return Ok(0);
    }
    let first = cursor
        .current_byte()
        .ok_or(JsonCountError::UnexpectedEof("end of buffer"))?;
    if first == b'"' {
        return Ok(u64::from(skip_string_nonempty(cursor).await?));
    }
    if first == b'{' || first == b'[' {
        return scan_container(cursor, bufs).await;
    }
    scan_bare_scalar(cursor, bufs, first).await
}

fn record_column_hit(
    fields: &mut Vec<ColumnResult>,
    index_by_key: &mut HashMap<String, usize>,
    bufs: &mut ScanBuffers,
    per_row: bool,
    raw_count: u64,
) {
    let count = if per_row {
        u64::from(raw_count > 0)
    } else {
        raw_count
    };
    let idx = if let Some(&idx) = index_by_key.get(&bufs.key) {
        idx
    } else {
        let idx = fields.len();
        let owned_key = bufs.key.clone();
        index_by_key.insert(owned_key.clone(), idx);
        fields.push(ColumnResult {
            key: owned_key,
            count: 0,
        });
        idx
    };
    let add = if per_row {
        u64::from(count > 0 && bufs.mark_counted_in_row(idx))
    } else {
        count
    };
    if let Some(col) = fields.get_mut(idx) {
        col.count += add;
    }
}

struct DelimSpec {
    close: u8,
    trailing: JsonCountError,
    expected: JsonCountError,
    eof: &'static str,
}

async fn expect_comma_or_close<F: FnMut(u64, u64), S: ChunkSource>(
    cur: &mut BlobCursor<F, S>,
    spec: DelimSpec,
) -> Result<bool, JsonCountError> {
    cur.skip_ws().await?;
    match cur.next_byte().await? {
        Some(b',') => {
            cur.skip_ws().await?;
            if cur.peek().await? == Some(spec.close) {
                return Err(spec.trailing);
            }
            Ok(true)
        }
        Some(b) if b == spec.close => Ok(false),
        Some(_) => Err(spec.expected),
        None => Err(JsonCountError::UnexpectedEof(spec.eof)),
    }
}

async fn scan_object<F: FnMut(u64, u64), S: ChunkSource>(
    cur: &mut BlobCursor<F, S>,
    fields: &mut Vec<ColumnResult>,
    index_by_key: &mut HashMap<String, usize>,
    bufs: &mut ScanBuffers,
    per_row: bool,
) -> Result<(), JsonCountError> {
    cur.skip_ws().await?;
    if cur.peek().await? == Some(b'}') {
        cur.next_byte().await?;
        return Ok(());
    }

    if per_row {
        bufs.begin_row();
    }
    loop {
        cur.skip_ws().await?;
        read_json_key_into(cur, &mut bufs.raw_key, &mut bufs.key).await?;
        cur.skip_ws().await?;
        if cur.next_byte().await? != Some(b':') {
            return Err(JsonCountError::ExpectedColon);
        }
        cur.skip_ws().await?;
        let raw_count = count_value_with_bufs(cur, bufs).await?;
        record_column_hit(fields, index_by_key, bufs, per_row, raw_count);

        if !expect_comma_or_close(
            cur,
            DelimSpec {
                close: b'}',
                trailing: JsonCountError::TrailingCommaInObject,
                expected: JsonCountError::ExpectedCommaOrBrace,
                eof: "missing closing '}'",
            },
        )
        .await?
        {
            return Ok(());
        }
    }
}

async fn scan_row_array<F: FnMut(u64, u64), S: ChunkSource>(
    cur: &mut BlobCursor<F, S>,
    fields: &mut Vec<ColumnResult>,
    index_by_key: &mut HashMap<String, usize>,
    bufs: &mut ScanBuffers,
) -> Result<(), JsonCountError> {
    cur.skip_ws().await?;
    if cur.peek().await? == Some(b']') {
        cur.next_byte().await?;
        return Ok(());
    }
    loop {
        cur.skip_ws().await?;
        if cur.next_byte().await? != Some(b'{') {
            return Err(JsonCountError::ExpectedTopLevelObject);
        }
        scan_object(cur, fields, index_by_key, bufs, true).await?;
        if !expect_comma_or_close(
            cur,
            DelimSpec {
                close: b']',
                trailing: JsonCountError::TrailingCommaInArray,
                expected: JsonCountError::ExpectedCommaOrBracket,
                eof: "missing closing ']'",
            },
        )
        .await?
        {
            return Ok(());
        }
    }
}

/// Scans a top-level JSON column object (`{...}`) or row array (`[{...}, ...]`).
///
/// # Errors
///
/// Returns [`JsonCountError`] on malformed RFC 8259 JSON, invalid string escapes/UTF-8, or I/O errors.
pub(crate) async fn scan_columns<F: FnMut(u64, u64), S: ChunkSource>(
    cur: &mut BlobCursor<F, S>,
) -> Result<Vec<ColumnResult>, JsonCountError> {
    cur.skip_ws().await?;
    let Some(open) = cur.next_byte().await? else {
        return Ok(Vec::new());
    };

    let mut fields = Vec::new();
    let mut index_by_key = HashMap::new();
    let mut bufs = ScanBuffers::default();

    match open {
        b'{' => scan_object(cur, &mut fields, &mut index_by_key, &mut bufs, false).await?,
        b'[' => scan_row_array(cur, &mut fields, &mut index_by_key, &mut bufs).await?,
        _ => return Err(JsonCountError::ExpectedTopLevelObject),
    }

    cur.skip_ws().await?;
    if cur.peek().await?.is_some() {
        return Err(JsonCountError::TrailingContent);
    }
    Ok(fields)
}

/// Runs `scan_columns` over any [`ChunkSource`] while reporting byte progress.
///
/// # Errors
///
/// Returns [`JsonCountError`] on malformed RFC 8259 JSON, invalid string escapes/UTF-8, or read errors.
pub async fn scan_source_with_progress<F: FnMut(u64, u64), S: ChunkSource>(
    source: S,
    on_progress: F,
) -> Result<Vec<ColumnResult>, JsonCountError> {
    let mut cur = BlobCursor::new(source, on_progress);
    scan_columns(&mut cur).await
}
