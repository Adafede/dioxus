// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! The two string scanners: one that reads a key out, one that skips a value.
//!
//! A module of their own because they are the same scan written twice — the
//! same escape grammar, the same chunk-boundary handling, the same
//! `pass_bound` — and because together they were most of `processing.rs`.
//! Being a module of its own is also what lets `cargo mutants` attribute their
//! tests to them, for the reason `mod escaping` and `mod streaming` give.
//!
//! The difference between them is what they are for, and it is the reason they
//! are two functions rather than one with a flag: `read_json_key` keeps the
//! bytes and hands them to [`super::unescape_json_string`], so a key is the
//! column name the browser would report; `skip_string_nonempty` keeps a single
//! flag and allocates nothing, because all it has to say is whether the string
//! had anything in it.

use super::{pass_bound, unescape_json_string};
#[cfg(any(test, target_arch = "wasm32"))]
use upload::{BlobCursor, ChunkSource, UploadError};

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
pub(crate) async fn read_json_key<F: FnMut(u64, u64), S: ChunkSource>(
    cursor: &mut BlobCursor<F, S>,
) -> Result<String, UploadError> {
    if cursor.next_byte().await? != Some(b'"') {
        return Err(UploadError::other("Expected opening quote for string"));
    }

    let mut raw = Vec::new();
    let mut escaped = false;
    for _ in 0..pass_bound(cursor) {
        let start = cursor.pos();

        // Indexed by an iterator rather than by a hand-kept `i`.
        //
        // `i += 1` in a loop condition is the shape that turns a scan into a
        // hang: every mutant that stops advancing it — `*=`, or `-=` where the
        // index is non-zero — leaves the loop reading one byte for ever, and
        // `cargo mutants` reported a dozen of those as timeouts on this file
        // before the loop stopped having a cursor of its own to lose. There is
        // no index here to stop moving; the only thing the loop can do is
        // finish or `break`.
        let (consumed, closed) = {
            let Some(tail) = cursor.buffer().get(start..) else {
                break;
            };
            let mut consumed = 0usize;
            let mut closed = false;
            for (i, &byte) in tail.iter().enumerate() {
                consumed = i + 1;
                match byte {
                    // The escaped byte belongs to the key, and it goes back into
                    // `raw` *with* its backslash, so that `raw` stays a faithful
                    // copy of the body as it appeared in the file and
                    // `unescape_json_string` below has the whole grammar to work
                    // with. Dropping the pair — which is what this arm did, and
                    // what the `b'\\'` arm did by not pushing the backslash
                    // either — meant `{"a\"b":1}` reported the column as `ab`,
                    // and `{"a\u0041b":1}` reported it as `ub`. Every column whose
                    // name contained an escaped character was mislabelled.
                    _ if escaped => {
                        escaped = false;
                        raw.push(b'\\');
                        raw.push(byte);
                    }
                    b'\\' => escaped = true,
                    b'"' => {
                        closed = true;
                        break;
                    }
                    _ => raw.push(byte),
                }
            }
            (consumed, closed)
        };
        if closed {
            // The key body *and* its closing quote. Advancing only 1 — which is
            // what this did — leaves the cursor sitting on the closing quote, so
            // the caller's colon check reads `"` and every non-empty key fails
            // with "Expected ':' after object key". It works for `{"":1}`, where
            // the body is empty and the two advances coincide, which is why
            // nothing noticed: no test could reach this function at all.
            cursor.advance(consumed);
            return Ok(unescape_json_string(&raw));
        }

        // Consume what was scanned before refilling. Without this the buffer is
        // never drained — `fill` drains `0..pos`, and `pos` has not moved — so the
        // next pass re-scans the same bytes and appends them to `raw` again: a key
        // split across a chunk boundary came back as `nn` for `"n"` and
        // `sststrstr` for `"str"`, and a key long enough to span many chunks grew
        // `raw` without bound. `skip_string_nonempty` has always had this line.
        cursor.advance(consumed);

        if !cursor.fill().await? {
            return Err(UploadError::other("Unexpected EOF while reading string"));
        }
    }

    Err(UploadError::other(
        "String key did not terminate within the document",
    ))
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
pub(crate) async fn skip_string_nonempty<F: FnMut(u64, u64), S: ChunkSource>(
    cursor: &mut BlobCursor<F, S>,
) -> Result<bool, UploadError> {
    if cursor.next_byte().await? != Some(b'"') {
        return Err(UploadError::other("Expected opening quote for string"));
    }

    let mut escaped = false;
    let mut any = false;
    for _ in 0..pass_bound(cursor) {
        let start = cursor.pos();

        // Iterator-indexed, for the reason `read_json_key` gives.
        //
        // `body` counts the bytes *before* the closing quote and `consumed`
        // counts the ones the cursor is handed, which is the same byte when the
        // string never closes and one more when it does. Collapsing the two
        // makes `""` — a body of nothing, which is the case the whole function
        // exists to report — read as a body of one.
        let (consumed, body, closed) = {
            let Some(tail) = cursor.buffer().get(start..) else {
                break;
            };
            let mut consumed = 0usize;
            let mut body = 0usize;
            let mut closed = false;
            for (i, &byte) in tail.iter().enumerate() {
                consumed = i + 1;
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
                body = consumed;
            }
            (consumed, body, closed)
        };

        if body > 0 {
            any = true;
        }
        cursor.advance(consumed);

        if closed {
            // `consumed` already includes the closing quote: it is `i + 1` at
            // the quote. The cursor is handed every byte the pass read, closing
            // quote included, and there is no second advance for it.
            return Ok(any);
        }

        if !cursor.fill().await? {
            return Err(UploadError::other("Unexpected EOF while reading string"));
        }
    }

    Err(UploadError::other(
        "String did not terminate within the document",
    ))
}
