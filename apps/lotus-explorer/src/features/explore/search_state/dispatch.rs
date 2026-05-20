// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::features::explore::actions::ExploreAction;
use dioxus::prelude::*;

use super::{ExploreState, reduce};

pub fn dispatch_explore_action(mut state: Signal<ExploreState>, action: ExploreAction) {
    let next = reduce(state.peek().clone(), action);
    if *state.peek() != next {
        *state.write() = next;
    }
}
