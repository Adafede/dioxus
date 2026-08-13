// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Lotus Knowledge Explorer CSS bundled as Rust submodules.

pub mod accessibility;
pub mod base;
pub mod curation;
pub mod form_controls;
pub mod layout_shell;
pub mod responsive;
pub mod tokens;

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

#[cfg(test)]
mod tests {
    use super::bundled_lotus_styles;

    #[test]
    fn bundled_lotus_styles_is_non_empty() {
        let css = bundled_lotus_styles();
        assert!(!css.is_empty());
        assert!(css.len() > 1000);
    }

    #[test]
    fn bundled_lotus_styles_contains_layout_shell() {
        let css = bundled_lotus_styles();
        assert!(
            css.contains("app-layout") || css.contains("layout"),
            "bundle must contain the app layout CSS"
        );
    }

    #[test]
    fn bundled_lotus_styles_contains_accessibility() {
        let css = bundled_lotus_styles();
        assert!(
            css.contains("focus") || css.contains("prefers-"),
            "bundle must contain accessibility CSS"
        );
    }
}
