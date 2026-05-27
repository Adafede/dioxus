// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::models::{BuildResult, BusyState, DataStats};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppState {
    pub session_id: Option<String>,
    pub stats: Option<DataStats>,
    pub build: Option<BuildResult>,
    pub busy: BusyState,
    pub error: Option<String>,
    pub status_message: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            session_id: None,
            stats: None,
            build: None,
            busy: BusyState::Idle,
            error: None,
            status_message: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_state_defaults_to_idle_without_data() {
        let state = AppState::default();
        assert_eq!(state.busy, BusyState::Idle);
        assert!(state.session_id.is_none());
        assert!(state.stats.is_none());
        assert!(state.build.is_none());
    }
}
