// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Query execution strategy — selects between the three search paths.
//!
//! Moving the branching logic here removes conditionals scattered across
//! `do_search` and makes the behavior matrix explicit and testable.

/// Controls which I/O path the search pipeline takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionStrategy {
    /// Try the REST API first; fall back to direct SPARQL if the API is
    /// unconfigured or returns an error.
    ApiFirst,
    /// Skip all queries; build the query string and return it for download.
    DownloadOnly,
}

impl ExecutionStrategy {
    /// Choose a strategy from the `direct_download` flag.
    ///
    /// When the user triggered a download-only mode (`direct_download = true`)
    /// we build the query but never fetch results.  Otherwise we prefer the
    /// fast REST API path (`ApiFirst`).
    pub fn resolve(direct_download: bool) -> Self {
        if direct_download {
            Self::DownloadOnly
        } else {
            Self::ApiFirst
        }
    }

    pub fn is_download_only(self) -> bool {
        self == Self::DownloadOnly
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_flag_selects_download_only() {
        assert_eq!(
            ExecutionStrategy::resolve(true),
            ExecutionStrategy::DownloadOnly
        );
    }

    #[test]
    fn normal_flag_selects_api_first() {
        assert_eq!(
            ExecutionStrategy::resolve(false),
            ExecutionStrategy::ApiFirst
        );
    }

    #[test]
    fn download_only_is_download_only() {
        assert!(ExecutionStrategy::DownloadOnly.is_download_only());
        assert!(!ExecutionStrategy::ApiFirst.is_download_only());
    }
}
