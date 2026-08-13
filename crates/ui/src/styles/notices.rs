// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared notice and alert styling system for notifications, warnings, etc.
//! Provides consistent styling across all apps for maximum reuse.

use crate::theme::StyleBuilder;

/// Base notice: flex container with border, padding, and gap.
pub fn notice_base_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .align_items("flex-start")
        .gap("8px")
        .padding("10px 12px")
        .border_radius("4px")
        .border("1px solid")
        .font_size("var(--fs-0)")
        .build()
}

/// Notice in dark mode: dark background with light border and text.
pub fn notice_dark_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .align_items("flex-start")
        .gap("8px")
        .padding("10px 12px")
        .border_radius("4px")
        .border("1px solid #444")
        .background_color("#1a1a1a")
        .color("#fff")
        .font_size("var(--fs-0)")
        .build()
}

/// Notice value text: word-break for long content with proper line height.
pub fn notice_value_style() -> String {
    StyleBuilder::new()
        .color("inherit")
        .property("word-break", "break-word")
        .property("line-height", "1.4")
        .build()
}

/// Notice dismiss button: close icon with opacity and no border.
pub fn notice_dismiss_style() -> String {
    StyleBuilder::new()
        .property("margin-left", "auto")
        .background_color("transparent")
        .border("0")
        .color("inherit")
        .cursor("pointer")
        .property("font-size", "18px")
        .property("line-height", "1")
        .padding("0 4px")
        .property("opacity", ".7")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notice_base_is_flex() {
        let style = notice_base_style();
        assert!(style.contains("display") && style.contains("flex"));
    }

    #[test]
    fn notice_value_breaks_words() {
        let style = notice_value_style();
        assert!(style.contains("word-break"));
    }

    #[test]
    fn notice_dismiss_has_no_border() {
        let style = notice_dismiss_style();
        assert!(style.contains("border") && style.contains("0"));
    }
}
