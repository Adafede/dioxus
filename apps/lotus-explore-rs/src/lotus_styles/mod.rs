// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus Knowledge Explorer CSS bundled as Rust submodules.

pub mod accessibility;
pub mod base;
pub mod curation;
pub mod form_controls;
pub mod layout_shell;
pub mod responsive;

pub fn bundled_lotus_styles() -> String {
    [
        base::css(),
        accessibility::css(),
        curation::css(),
        form_controls::css(),
        layout_shell::css(),
        responsive::css(),
    ]
    .join("\n\n")
}
