// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Portable streaming RFC 8259 JSON non-null field counter and result sorter.
//!
//! Counts non-null values per top-level key over any [`stream_io::ChunkSource`]
//! with bounded memory and reusable scratch buffers across array rows.
//!
//! # Example
//!
//! ```rust
//! use futures_executor::block_on;
//! use json_count_core::{ColumnResult, scan_source_with_progress};
//! use stream_io::SliceSource;
//!
//! let doc = br#"{"a":1,"b":[null,"ok"]}"#;
//! let cols = block_on(scan_source_with_progress(
//!     SliceSource::new(doc),
//!     doc.len() as u64,
//!     |_, _| {},
//! ));
//! assert_eq!(
//!     cols,
//!     Ok(vec![
//!         ColumnResult { key: "a".to_string(), count: 1 },
//!         ColumnResult { key: "b".to_string(), count: 1 },
//!     ])
//! );
//! ```

#![allow(clippy::future_not_send)]
#![warn(missing_docs)]

mod container;
mod error;
mod model;
mod number;
mod scanner;
mod strings;
mod unescape;

pub use error::JsonCountError;
pub use model::{
    ColumnResult, SortColumn, SortDirection, SortSpec, aria_sort_for, next_sort_state,
    progress_message, sorted_results,
};
pub use scanner::{scan_columns, scan_source_with_progress};
pub use unescape::unescape_json_string;

#[cfg(test)]
mod tests;
