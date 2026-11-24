// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! Streaming container (`{...}` / `[...]`) and bare-scalar value scanner.

use std::collections::HashSet;

use stream_io::{BlobCursor, ChunkSource};

use crate::error::JsonCountError;
use crate::number::{push_scalar_byte, validate_bare_scalar};
use crate::strings::pass_bound;

/// Reusable scratch buffers carried across rows of a streamed JSON document.
#[derive(Debug, Default)]
pub(crate) struct ScanBuffers {
    pub(crate) raw_key: Vec<u8>,
    pub(crate) key: String,
    pub(crate) scalar: Vec<u8>,
    pub(crate) stack: Vec<u8>,
    pub(crate) counted_in_row: HashSet<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContainerState {
    ExpectValue,
    InString { has_content: bool, escaped: bool },
    InToken,
}

const fn step_string(
    byte: u8,
    has_content: bool,
    escaped: bool,
    count: &mut u64,
) -> ContainerState {
    if escaped {
        ContainerState::InString {
            has_content: true,
            escaped: false,
        }
    } else if byte == b'\\' {
        ContainerState::InString {
            has_content: true,
            escaped: true,
        }
    } else if byte == b'"' {
        if has_content {
            *count += 1;
        }
        ContainerState::ExpectValue
    } else {
        ContainerState::InString {
            has_content: true,
            escaped: false,
        }
    }
}

const fn is_scalar_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        b' ' | b'\t' | b'\n' | b'\r' | b',' | b':' | b'}' | b']'
    )
}

fn step_expect_value(
    byte: u8,
    stack: &mut Vec<u8>,
    scalar: &mut Vec<u8>,
) -> Result<(ContainerState, bool), JsonCountError> {
    match byte {
        b'"' => Ok((
            ContainerState::InString {
                has_content: false,
                escaped: false,
            },
            false,
        )),
        b'{' => {
            stack.push(b'}');
            Ok((ContainerState::ExpectValue, false))
        }
        b'[' => {
            stack.push(b']');
            Ok((ContainerState::ExpectValue, false))
        }
        b'}' | b']' => {
            if stack.pop() != Some(byte) {
                return Err(JsonCountError::MismatchedClosingDelimiter);
            }
            Ok((ContainerState::ExpectValue, stack.is_empty()))
        }
        b':' | b',' | b' ' | b'\t' | b'\n' | b'\r' => Ok((ContainerState::ExpectValue, false)),
        _ => {
            scalar.clear();
            push_scalar_byte(scalar, byte)?;
            Ok((ContainerState::InToken, false))
        }
    }
}

fn scan_container_chunk(
    tail: &[u8],
    state: &mut ContainerState,
    bufs: &mut ScanBuffers,
    count: &mut u64,
) -> Result<(usize, bool), JsonCountError> {
    let mut consumed = 0usize;
    for (i, &byte) in tail.iter().enumerate() {
        consumed = i + 1;
        match *state {
            ContainerState::InString {
                has_content,
                escaped,
            } => {
                *state = step_string(byte, has_content, escaped, count);
            }
            ContainerState::InToken => {
                if is_scalar_delimiter(byte) {
                    *count += validate_bare_scalar(&bufs.scalar)?;
                    bufs.scalar.clear();
                    let (next_state, closed) =
                        step_expect_value(byte, &mut bufs.stack, &mut bufs.scalar)?;
                    *state = next_state;
                    if closed {
                        return Ok((consumed, true));
                    }
                } else {
                    push_scalar_byte(&mut bufs.scalar, byte)?;
                }
            }
            ContainerState::ExpectValue => {
                let (next_state, closed) =
                    step_expect_value(byte, &mut bufs.stack, &mut bufs.scalar)?;
                *state = next_state;
                if closed {
                    return Ok((consumed, true));
                }
            }
        }
    }
    Ok((consumed, false))
}

/// Counts non-null leaf values inside a `{...}` or `[...]` container.
pub(crate) async fn scan_container<F: FnMut(u64, u64), S: ChunkSource>(
    cursor: &mut BlobCursor<F, S>,
    bufs: &mut ScanBuffers,
) -> Result<u64, JsonCountError> {
    let open = cursor
        .current_byte()
        .ok_or(JsonCountError::ExpectedTopLevelObject)?;
    let initial_close = if open == b'{' { b'}' } else { b']' };
    cursor.advance(1);

    bufs.stack.clear();
    bufs.stack.push(initial_close);
    bufs.scalar.clear();
    let mut state = ContainerState::ExpectValue;
    let mut count = 0u64;

    for _ in 0..pass_bound(cursor) {
        let Some(tail) = cursor.buffer().get(cursor.pos()..) else {
            break;
        };
        let (consumed, closed) = scan_container_chunk(tail, &mut state, bufs, &mut count)?;
        cursor.advance(consumed);
        if closed {
            return Ok(count);
        }
        if !cursor.fill().await? {
            return Err(JsonCountError::UnexpectedEof(
                "while scanning nested JSON value",
            ));
        }
    }

    Err(JsonCountError::Unterminated("Nested JSON value"))
}

/// Scans a top-level bare scalar token starting with `first`.
pub(crate) async fn scan_bare_scalar<F: FnMut(u64, u64), S: ChunkSource>(
    cursor: &mut BlobCursor<F, S>,
    bufs: &mut ScanBuffers,
    first: u8,
) -> Result<u64, JsonCountError> {
    bufs.scalar.clear();
    push_scalar_byte(&mut bufs.scalar, first)?;
    cursor.advance(1);

    for _ in 0..pass_bound(cursor) {
        let Some(tail) = cursor.buffer().get(cursor.pos()..) else {
            break;
        };
        let mut consumed = 0usize;
        let mut at_delimiter = false;
        for &byte in tail {
            if is_scalar_delimiter(byte) {
                at_delimiter = true;
                break;
            }
            push_scalar_byte(&mut bufs.scalar, byte)?;
            consumed += 1;
        }
        cursor.advance(consumed);
        if at_delimiter || !cursor.fill().await? {
            return validate_bare_scalar(&bufs.scalar);
        }
    }

    Err(JsonCountError::Unterminated("Scalar"))
}
