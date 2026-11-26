// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! Structured errors for streaming JSON field counting.

use stream_io::StreamError;
use thiserror::Error;

/// Error returned when streaming or parsing a JSON document.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum JsonCountError {
    /// Underlying byte stream read failure.
    #[error(transparent)]
    Stream(#[from] StreamError),

    /// Document did not start with `{` or `[` (or array item was not `{`).
    #[error("Expected a top-level JSON object")]
    ExpectedTopLevelObject,

    /// Missing `:` between an object key and its value.
    #[error("Expected ':' after object key")]
    ExpectedColon,

    /// Expected `"` at the start of a JSON string.
    #[error("Expected opening quote for string")]
    ExpectedOpeningQuote,

    /// Object key exceeded the 1 MiB safety cap.
    #[error("JSON object key exceeds maximum length")]
    KeyTooLong,

    /// Bare scalar exceeded the 64 KiB safety cap.
    #[error("JSON scalar exceeds maximum length")]
    ScalarTooLong,

    /// Bare scalar was not `null`, `true`, `false`, or a valid RFC 8259 number.
    #[error("Invalid JSON literal or number")]
    InvalidBareScalar,

    /// Escape sequence inside a JSON string was invalid or incomplete.
    #[error("Invalid escape sequence in JSON string")]
    InvalidEscape,

    /// Unescaped ASCII control character (`< 0x20`) inside a JSON string.
    #[error("Unescaped control character in JSON string")]
    UnescapedControlChar,

    /// Byte sequence inside a JSON string was not valid UTF-8.
    #[error("Invalid UTF-8 sequence in JSON string")]
    InvalidUtf8,

    /// Closing bracket/brace did not match the active container.
    #[error("Mismatched closing delimiter in JSON")]
    MismatchedClosingDelimiter,

    /// Trailing comma before `}` in an object.
    #[error("Trailing comma in JSON object")]
    TrailingCommaInObject,

    /// Unexpected token after an object entry.
    #[error("Expected ',' or '}}' in JSON object")]
    ExpectedCommaOrBrace,

    /// Trailing comma before `]` in an array.
    #[error("Trailing comma in JSON array")]
    TrailingCommaInArray,

    /// Unexpected token after an array item.
    #[error("Expected ',' or ']' in JSON array")]
    ExpectedCommaOrBracket,

    /// Non-whitespace bytes remained after the root value closed.
    #[error("Unexpected trailing content after JSON document")]
    TrailingContent,

    /// Stream ended unexpectedly inside a construct.
    #[error("Unexpected EOF: {0}")]
    UnexpectedEof(&'static str),

    /// Construct exceeded the pass bound without terminating.
    #[error("{0} did not terminate within the document")]
    Unterminated(&'static str),
}
