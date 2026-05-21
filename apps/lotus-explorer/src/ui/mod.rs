// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! UI composition primitives used by live components.

pub mod a11y_contract;
mod a11y_smoke;

/// A content-phase enumeration for better state-driven UI rendering.
///
/// Replaces ad-hoc if-else chains in container components with explicit phase tracking.
/// This improves readability and ensures exhaustive handling of all UI states.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContentPhase {
    /// Initial welcome/empty state before any search.
    Welcome,
    /// Active loading state (search in progress).
    Loading,
    /// Error state with associated message.
    Error,
    /// Empty results (search completed, no matches).
    Empty,
    /// Content loaded and ready to display.
    Loaded,
    /// Special download-only mode (no preview).
    DownloadOnly,
}

impl ContentPhase {
    /// Determine phase from lifecycle state selectors.
    pub const fn from_lifecycle(
        loading: bool,
        error: bool,
        searched_once: bool,
        download_only: bool,
        has_entries: bool,
    ) -> Self {
        if loading {
            Self::Loading
        } else if error {
            Self::Error
        } else if !searched_once {
            Self::Welcome
        } else if download_only && !has_entries {
            Self::DownloadOnly
        } else if !has_entries {
            Self::Empty
        } else {
            Self::Loaded
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_phase_welcome_when_not_searched() {
        let phase = ContentPhase::from_lifecycle(false, false, false, false, false);
        assert_eq!(phase, ContentPhase::Welcome);
    }

    #[test]
    fn content_phase_loading_takes_priority() {
        let phase = ContentPhase::from_lifecycle(true, false, true, false, true);
        assert_eq!(phase, ContentPhase::Loading);
    }

    #[test]
    fn content_phase_error_takes_precedence_over_empty() {
        let phase = ContentPhase::from_lifecycle(false, true, true, false, false);
        assert_eq!(phase, ContentPhase::Error);
    }

    #[test]
    fn content_phase_empty_when_no_results_after_search() {
        let phase = ContentPhase::from_lifecycle(false, false, true, false, false);
        assert_eq!(phase, ContentPhase::Empty);
    }

    #[test]
    fn content_phase_loaded_when_entries_exist() {
        let phase = ContentPhase::from_lifecycle(false, false, true, false, true);
        assert_eq!(phase, ContentPhase::Loaded);
    }
}
