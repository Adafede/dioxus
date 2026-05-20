use crate::models::SearchCriteria;

use super::super::UiChromeState;

pub(super) fn search_requested(state: &mut UiChromeState, criteria_snapshot: SearchCriteria) {
    state.executed_criteria = criteria_snapshot;
    state.mobile_filters_open = false;
}

pub(super) fn toggle_mobile_filters(state: &mut UiChromeState) {
    state.mobile_filters_open = !state.mobile_filters_open;
}
