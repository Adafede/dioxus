// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Structured errors for chunked byte and line streaming.

use thiserror::Error;

/// Error returned by [`crate::ChunkSource`], [`crate::BlobCursor`], and [`crate::BlobLines`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum StreamError {
    /// Reading a chunk from the underlying source failed.
    #[error("stream read error: {0}")]
    Io(String),

    /// A requested byte offset could not be represented as `usize`.
    #[error("chunk offset {offset} does not fit a usize")]
    OffsetOutOfRange {
        /// The out-of-range byte offset.
        offset: u64,
    },

    /// A line exceeded the configured maximum byte length.
    #[error("line exceeds maximum length of {max_bytes} bytes")]
    LineTooLong {
        /// Configured line length limit in bytes.
        max_bytes: usize,
    },

    /// A caller-supplied streaming validation failure.
    #[error("{0}")]
    Other(String),
}

impl StreamError {
    /// Creates an I/O read failure with context.
    #[must_use]
    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(msg.into())
    }

    /// Creates a custom streaming error message.
    #[must_use]
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::StreamError;

    #[test]
    fn error_messages_are_clear() {
        assert_eq!(
            StreamError::io("disk read failed").to_string(),
            "stream read error: disk read failed"
        );
        assert_eq!(
            StreamError::OffsetOutOfRange { offset: 99 }.to_string(),
            "chunk offset 99 does not fit a usize"
        );
        assert_eq!(
            StreamError::LineTooLong { max_bytes: 128 }.to_string(),
            "line exceeds maximum length of 128 bytes"
        );
        assert_eq!(StreamError::other("bad magic").to_string(), "bad magic");
    }
}
