// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the index project

//! Application card backed by the shared [`ui::prelude::Card`] component.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus component macro generates a Props helper empty enum"
)]

use super::AppInfo;
use dioxus::prelude::*;
use ui::prelude::Card;

/// Card for a single application: `<article>` wrapping an `<h2>` link and a description.
#[component]
pub(crate) fn AppCard(app: AppInfo) -> Element {
    rsx! {
        Card {
            title: app.title.to_string(),
            href: Some(app.path.to_string()),
            "{app.description}"
        }
    }
}
