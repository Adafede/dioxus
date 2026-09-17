use crate::models::SearchCriteria;

use super::super::UiChromeState;

pub(super) fn search_requested(state: &mut UiChromeState, criteria_snapshot: SearchCriteria) {
    state.executed_criteria = criteria_snapshot;
}
