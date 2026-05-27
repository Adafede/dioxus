// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::models::BusyState;

#[test]
fn busy_state_labels_are_stable() {
    assert_eq!(BusyState::Idle.label(), "Ready");
    assert_eq!(BusyState::Fetching.label(), "Fetching data from Wikidata…");
    assert_eq!(
        BusyState::Building.label(),
        "Building tree previews and downloads…"
    );
}
