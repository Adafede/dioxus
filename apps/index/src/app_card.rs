// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the index project

//! Application card backed by the shared [`ui::Card`] component.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus component macro generates a Props helper empty enum"
)]

use dioxus::prelude::*;
use ui::Card;

/// Metadata for an application card.
///
/// Used by [`AppCard`] to render application information.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AppInfo {
    /// Unique identifier, used as the Dioxus list key in [`AppCard`].
    pub(crate) id: &'static str,
    /// Application display title (e.g., "Lipid Selecto-rs").
    pub(crate) title: &'static str,
    /// Path under the deployed root (e.g., "./lipid-selecto-rs/").
    pub(crate) path: &'static str,
    /// Short, clear description of functionality.
    pub(crate) description: &'static str,
}

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
