// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

use futures_executor::block_on;
use stream_io::{BlobCursor, SliceSource};

use crate::JsonCountError;
use crate::scanner::{count_value, scan_columns};

mod boundaries;
mod escaping;
mod sorting;
mod streaming;

pub(super) fn cursor(bytes: &[u8], chunk: usize) -> BlobCursor<fn(u64, u64), SliceSource<'_>> {
    let on_progress: fn(u64, u64) = |_, _| {};
    BlobCursor::new(SliceSource::new(bytes), bytes.len() as u64, on_progress).with_chunk_size(chunk)
}

pub(super) fn count_str(input: &str) -> Result<u64, JsonCountError> {
    let mut cur = cursor(input.as_bytes(), 64);
    block_on(count_value(&mut cur))
}

pub(super) fn columns(json: &str, chunk: usize) -> Result<Vec<(String, u64)>, JsonCountError> {
    let mut cur = cursor(json.as_bytes(), chunk);
    block_on(scan_columns(&mut cur))
        .map(|cols| cols.into_iter().map(|c| (c.key, c.count)).collect())
}
