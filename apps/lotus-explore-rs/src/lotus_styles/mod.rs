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
    let base_css = base::CSS
        .lines()
        .filter(|line| !line.trim_start().starts_with("@import url("))
        .collect::<Vec<_>>()
        .join("\n");

    [
        base_css,
        accessibility::CSS.to_string(),
        curation::CSS.to_string(),
        form_controls::CSS.to_string(),
        layout_shell::CSS.to_string(),
        responsive::CSS.to_string(),
    ]
    .join("\n\n")
}
