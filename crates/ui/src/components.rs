// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Reusable Dioxus components for unified UI across all applications.
//!
//! All components use pure Rust styling via [`crate::theme`], no external CSS files.

pub(crate) mod button;
pub(crate) mod card;
pub(crate) mod footer;
pub(crate) mod header;
pub(crate) mod notice;
pub(crate) mod segmented_control;
pub(crate) mod simple_analytics_badge;
pub(crate) mod upload_zone;

#[cfg(test)]
pub(super) fn render_debug(app: fn() -> dioxus::prelude::Element) -> String {
    let mut dom = dioxus::prelude::VirtualDom::new(app);
    format!("{:?}", dom.rebuild_to_vec())
}
