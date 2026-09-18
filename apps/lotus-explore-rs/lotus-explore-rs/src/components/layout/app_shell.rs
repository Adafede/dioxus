// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! App shell component with proper layout structure.
//!
//! This provides the main page structure with skip link for accessibility.

use dioxus::prelude::*;

/// Top-level application shell with proper accessibility features
#[derive(Props, PartialEq)]
pub struct AppShellProps {
    children: Element,
}

#[component]
pub fn AppShell(children: Element) -> Element {
    rsx! {
        div {
            class: "app-shell",
            {children}
        }
    }
}

/// Skip link for keyboard navigation
#[component]
pub fn SkipLink(destination: &'static str, label: &'static str) -> Element {
    rsx! {
        a {
            href: destination,
            class: "skip-link",
            "{label}"
        }
    }
}
