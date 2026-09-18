// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared utility styles for accessibility, typography, and common patterns.

use crate::ui::theme::StyleBuilder;

/// Screen-reader only text: hidden from visual display but readable by assistive tech.
pub fn sr_only_style() -> String {
    StyleBuilder::new()
        .property("position", "absolute")
        .property("width", "1px")
        .property("height", "1px")
        .property("padding", "0")
        .property("margin", "-1px")
        .property("overflow", "hidden")
        .property("white-space", "nowrap")
        .property("border", "0")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sr_only_is_hidden() {
        let style = sr_only_style();
        assert!(style.contains("width") && style.contains("1px"));
    }
}
