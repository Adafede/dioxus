// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Accessibility-focused styles for keyboard navigation, focus indicators, and motion preferences.

pub mod focus;
pub mod keyboard;
pub mod motion;

pub fn css() -> String {
    [focus::css(), keyboard::css(), motion::css()].join("\n\n")
}
