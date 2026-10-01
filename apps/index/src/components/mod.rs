// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the index project

//! UI components for the landing page.

mod app_card;

pub(crate) use app_card::AppCard;

/// Metadata for an application card.
///
/// Used by [`AppCard`] to render application information.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AppInfo {
    /// Unique identifier, used as the Dioxus list key in [`AppCard`].
    pub(crate) id: &'static str,
    /// Emoji and title (e.g., "🪷 LOTUS Wikidata Explorer").
    pub(crate) title: &'static str,
    /// Relative path to application (e.g., "./lotus-explore-rs/").
    pub(crate) path: &'static str,
    /// Short, clear description of functionality.
    pub(crate) description: &'static str,
}
