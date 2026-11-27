// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! Streaming container (`{...}` / `[...]`) and bare-scalar value scanner.

use stream_io::{BlobCursor, ChunkSource};

use crate::error::JsonCountError;
use crate::number::{push_scalar_byte, validate_bare_scalar};
use crate::unescape::StringValidator;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ContainerFrame {
    ObjectExpectKeyOrEnd,
    ObjectExpectKey,
    ObjectExpectColon,
    ObjectExpectValue,
    ObjectExpectCommaOrEnd,
    ArrayExpectValueOrEnd,
    ArrayExpectValue,
    ArrayExpectCommaOrEnd,
}

/// Reusable scratch buffers carried across rows of a streamed JSON document.
#[derive(Debug, Default)]
pub(crate) struct ScanBuffers {
    pub(crate) raw_key: Vec<u8>,
    pub(crate) key: String,
    pub(crate) scalar: Vec<u8>,
    pub(crate) stack: Vec<ContainerFrame>,
    row_seen: Vec<u64>,
    row_gen: u64,
}

impl ScanBuffers {
    pub(crate) fn begin_row(&mut self) {
        self.row_gen = self.row_gen.wrapping_add(1);
        if self.row_gen == 0 {
            self.row_seen.fill(0);
            self.row_gen = 1;
        }
    }

    pub(crate) fn mark_counted_in_row(&mut self, idx: usize) -> bool {
        if idx >= self.row_seen.len() {
            self.row_seen.resize(idx + 1, 0);
        }
        if let Some(slot) = self.row_seen.get_mut(idx)
            && *slot != self.row_gen
        {
            *slot = self.row_gen;
            return true;
        }
        false
    }
}

pub(crate) async fn drive_cursor_chunks<F, S, T>(
    cursor: &mut BlobCursor<F, S>,
    mut on_chunk: impl FnMut(&[u8]) -> Result<(usize, Option<T>), JsonCountError>,
) -> Result<Option<T>, JsonCountError>
where
    F: FnMut(u64, u64),
    S: ChunkSource,
{
    loop {
        let tail = cursor.remaining();
        let (consumed, finished) = on_chunk(tail)?;
        cursor.advance(consumed);
        if finished.is_some() {
            return Ok(finished);
        }
        if !cursor.fill().await? {
            return Ok(None);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContainerState {
    Container,
    String {
        is_key: bool,
        has_content: bool,
        validator: StringValidator,
    },
    Token,
}

const fn is_scalar_delimiter(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b',' | b'}' | b']')
}

const fn is_json_ws(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
}

const fn mark_value_complete(stack: &mut [ContainerFrame]) {
    if let Some(frame) = stack.last_mut() {
        *frame = match *frame {
            ContainerFrame::ObjectExpectValue => ContainerFrame::ObjectExpectCommaOrEnd,
            ContainerFrame::ArrayExpectValueOrEnd | ContainerFrame::ArrayExpectValue => {
                ContainerFrame::ArrayExpectCommaOrEnd
            }
            other => other,
        };
    }
}

fn start_value(
    byte: u8,
    stack: &mut Vec<ContainerFrame>,
    scalar: &mut Vec<u8>,
) -> Result<ContainerState, JsonCountError> {
    match byte {
        b'"' => Ok(ContainerState::String {
            is_key: false,
            has_content: false,
            validator: StringValidator::default(),
        }),
        b'{' => {
            stack.push(ContainerFrame::ObjectExpectKeyOrEnd);
            Ok(ContainerState::Container)
        }
        b'[' => {
            stack.push(ContainerFrame::ArrayExpectValueOrEnd);
            Ok(ContainerState::Container)
        }
        b'}' | b']' => Err(JsonCountError::MismatchedClosingDelimiter),
        b',' | b':' => Err(JsonCountError::InvalidBareScalar),
        _ => {
            scalar.clear();
            push_scalar_byte(scalar, byte)?;
            Ok(ContainerState::Token)
        }
    }
}

fn close_top_frame(stack: &mut Vec<ContainerFrame>) -> (ContainerState, bool) {
    stack.pop();
    if stack.is_empty() {
        return (ContainerState::Container, true);
    }
    mark_value_complete(stack);
    (ContainerState::Container, false)
}

fn step_in_container(
    byte: u8,
    stack: &mut Vec<ContainerFrame>,
    scalar: &mut Vec<u8>,
) -> Result<(ContainerState, bool), JsonCountError> {
    if is_json_ws(byte) {
        return Ok((ContainerState::Container, false));
    }
    let Some(&top) = stack.last() else {
        return Err(JsonCountError::MismatchedClosingDelimiter);
    };
    match top {
        ContainerFrame::ObjectExpectKeyOrEnd => match byte {
            b'"' => Ok((
                ContainerState::String {
                    is_key: true,
                    has_content: false,
                    validator: StringValidator::default(),
                },
                false,
            )),
            b'}' => Ok(close_top_frame(stack)),
            _ => Err(JsonCountError::ExpectedOpeningQuote),
        },
        ContainerFrame::ObjectExpectKey => match byte {
            b'"' => Ok((
                ContainerState::String {
                    is_key: true,
                    has_content: false,
                    validator: StringValidator::default(),
                },
                false,
            )),
            b'}' => Err(JsonCountError::TrailingCommaInObject),
            _ => Err(JsonCountError::ExpectedOpeningQuote),
        },
        ContainerFrame::ObjectExpectColon => {
            if byte != b':' {
                return Err(JsonCountError::ExpectedColon);
            }
            if let Some(frame) = stack.last_mut() {
                *frame = ContainerFrame::ObjectExpectValue;
            }
            Ok((ContainerState::Container, false))
        }
        ContainerFrame::ObjectExpectValue | ContainerFrame::ArrayExpectValue => {
            if byte == b']' && matches!(top, ContainerFrame::ArrayExpectValue) {
                return Err(JsonCountError::TrailingCommaInArray);
            }
            Ok((start_value(byte, stack, scalar)?, false))
        }
        ContainerFrame::ArrayExpectValueOrEnd => {
            if byte == b']' {
                return Ok(close_top_frame(stack));
            }
            Ok((start_value(byte, stack, scalar)?, false))
        }
        ContainerFrame::ObjectExpectCommaOrEnd => match byte {
            b',' => {
                if let Some(frame) = stack.last_mut() {
                    *frame = ContainerFrame::ObjectExpectKey;
                }
                Ok((ContainerState::Container, false))
            }
            b'}' => Ok(close_top_frame(stack)),
            b']' => Err(JsonCountError::MismatchedClosingDelimiter),
            _ => Err(JsonCountError::ExpectedCommaOrBrace),
        },
        ContainerFrame::ArrayExpectCommaOrEnd => match byte {
            b',' => {
                if let Some(frame) = stack.last_mut() {
                    *frame = ContainerFrame::ArrayExpectValue;
                }
                Ok((ContainerState::Container, false))
            }
            b']' => Ok(close_top_frame(stack)),
            b'}' => Err(JsonCountError::MismatchedClosingDelimiter),
            _ => Err(JsonCountError::ExpectedCommaOrBracket),
        },
    }
}

fn scan_container_chunk(
    tail: &[u8],
    state: &mut ContainerState,
    bufs: &mut ScanBuffers,
    count: &mut u64,
) -> Result<(usize, bool), JsonCountError> {
    let mut i = 0usize;
    while i < tail.len() {
        match state {
            ContainerState::String {
                is_key,
                has_content,
                validator,
            } => {
                let rest = tail.get(i..).unwrap_or_default();
                let run = validator.plain_ascii_run(rest);
                if run > 0 {
                    *has_content = true;
                    i += run;
                }
                let Some(&byte) = tail.get(i) else {
                    break;
                };
                i += 1;
                if validator.step(byte)? {
                    if *is_key {
                        if let Some(frame) = bufs.stack.last_mut() {
                            *frame = ContainerFrame::ObjectExpectColon;
                        }
                    } else {
                        if *has_content {
                            *count += 1;
                        }
                        mark_value_complete(&mut bufs.stack);
                    }
                    *state = ContainerState::Container;
                } else {
                    *has_content = true;
                }
            }
            ContainerState::Token => {
                let Some(&byte) = tail.get(i) else {
                    break;
                };
                i += 1;
                if is_scalar_delimiter(byte) {
                    *count += validate_bare_scalar(&bufs.scalar)?;
                    bufs.scalar.clear();
                    mark_value_complete(&mut bufs.stack);
                    let (next_state, closed) =
                        step_in_container(byte, &mut bufs.stack, &mut bufs.scalar)?;
                    *state = next_state;
                    if closed {
                        return Ok((i, true));
                    }
                } else {
                    push_scalar_byte(&mut bufs.scalar, byte)?;
                }
            }
            ContainerState::Container => {
                let Some(&byte) = tail.get(i) else {
                    break;
                };
                i += 1;
                let (next_state, closed) =
                    step_in_container(byte, &mut bufs.stack, &mut bufs.scalar)?;
                *state = next_state;
                if closed {
                    return Ok((i, true));
                }
            }
        }
    }
    Ok((i, false))
}

/// Counts non-null leaf values inside a `{...}` or `[...]` container.
pub(crate) async fn scan_container<F: FnMut(u64, u64), S: ChunkSource>(
    cursor: &mut BlobCursor<F, S>,
    bufs: &mut ScanBuffers,
) -> Result<u64, JsonCountError> {
    let open = cursor
        .current_byte()
        .ok_or(JsonCountError::ExpectedTopLevelObject)?;
    let initial_frame = if open == b'{' {
        ContainerFrame::ObjectExpectKeyOrEnd
    } else {
        ContainerFrame::ArrayExpectValueOrEnd
    };
    cursor.advance(1);

    bufs.stack.clear();
    bufs.stack.push(initial_frame);
    bufs.scalar.clear();
    let mut state = ContainerState::Container;
    let mut count = 0u64;

    drive_cursor_chunks(cursor, |tail| {
        let (consumed, closed) = scan_container_chunk(tail, &mut state, bufs, &mut count)?;
        Ok((consumed, closed.then_some(count)))
    })
    .await?
    .ok_or(JsonCountError::UnexpectedEof(
        "while scanning nested JSON value",
    ))
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

    if let Some(count) = drive_cursor_chunks(cursor, |tail| {
        for (i, &byte) in tail.iter().enumerate() {
            if is_scalar_delimiter(byte) {
                let count = validate_bare_scalar(&bufs.scalar)?;
                return Ok((i, Some(count)));
            }
            push_scalar_byte(&mut bufs.scalar, byte)?;
        }
        Ok((tail.len(), None))
    })
    .await?
    {
        Ok(count)
    } else {
        validate_bare_scalar(&bufs.scalar)
    }
}
