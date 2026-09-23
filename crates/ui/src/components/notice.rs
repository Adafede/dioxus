// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared notice bar component.

use crate::theme::{ColorScheme, Radius, Shadow, Spacing, StyleBuilder, Typography};
use dioxus::prelude::*;

/// Visual tone for a notice bar.
#[allow(clippy::module_name_repetitions)] // UI component type intentionally matches module name
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeTone {
    /// Neutral informational tone (no specific semantic meaning).
    Neutral,
    /// Informational tone (blue/cyan themed).
    Info,
    /// Success tone (green themed).
    Success,
    /// Warning tone (yellow/orange themed).
    Warning,
    /// Danger/alert tone (red themed).
    Danger,
}

/// Properties for the [`NoticeBar`] component.
#[allow(clippy::module_name_repetitions)] // UI component type intentionally matches module name
#[derive(Clone, Props, Debug, PartialEq)]
pub struct NoticeBarProps {
    /// The main label text displayed in the notice bar.
    pub label: String,
    /// Visual tone/style of the notice bar.
    #[props(default = NoticeTone::Neutral)]
    pub tone: NoticeTone,
    /// ARIA role attribute for accessibility.
    #[props(default = "status")]
    pub role: &'static str,
    /// ARIA live region setting for dynamic updates.
    #[props(default = "polite")]
    pub aria_live: &'static str,
    /// Whether to use dark theme colors.
    #[props(default = false)]
    pub dark: bool,
    /// CSS margin for the outer container.
    #[props(default = "10px 22px 0")]
    pub margin: &'static str,
    /// CSS padding for the inner content area.
    #[props(default = "9px 12px")]
    pub padding: &'static str,
    /// Optional trailing content (e.g., icons, buttons).
    #[props(default)]
    pub trailing: Option<Element>,
    /// Optional child elements to display in the notice body.
    #[props(default)]
    pub children: Option<Element>,
}

/// A notice bar component for displaying informational messages with visual tone styling.
///
/// Supports neutral, info, success, warning, and danger tones with customizable
/// content and styling.
#[component]
pub fn NoticeBar(props: NoticeBarProps) -> Element {
    let colors = if props.dark {
        ColorScheme::DARK
    } else {
        ColorScheme::LIGHT
    };
    let tone_color = match props.tone {
        NoticeTone::Neutral => colors.accent,
        NoticeTone::Info => colors.blue,
        NoticeTone::Success => colors.green,
        NoticeTone::Warning => colors.yellow,
        NoticeTone::Danger => colors.red,
    };
    let border_color = format!("color-mix(in srgb, {} 24%, {})", tone_color, colors.border);
    let label_background = format!("color-mix(in srgb, {} 12%, {})", tone_color, colors.bg2);
    let outer_background = format!("color-mix(in srgb, {} 4%, {})", tone_color, colors.bg2);

    let outer_style = StyleBuilder::new()
        .margin(props.margin)
        .padding(props.padding)
        .display("flex")
        .flex_direction("row")
        .flex_wrap("wrap")
        .align_items("center")
        .gap(Spacing::SM)
        .border(&format!("1px solid {border_color}"))
        .border_left(&format!("4px solid {tone_color}"))
        .border_radius(Radius::MD)
        .background_color(&outer_background)
        .box_shadow(Shadow::XS)
        .font_size(Typography::UI)
        .build();

    let label_style = StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .padding("2px 8px")
        .border_radius("999px")
        .background_color(&label_background)
        .color(tone_color)
        .font_size(Typography::LABEL)
        .font_weight("700")
        .property("letter-spacing", "0.08em")
        .property("text-transform", "uppercase")
        .property("flex-shrink", "0")
        .property("white-space", "nowrap")
        .build();

    let body_style = StyleBuilder::new()
        .display("flex")
        .flex_direction("row")
        .flex_wrap("wrap")
        .align_items("center")
        .gap(Spacing::SM)
        .property("min-width", "0")
        .property("flex", "1")
        .color(colors.text)
        .build();

    let trailing_style = StyleBuilder::new()
        .display("flex")
        .flex_direction("row")
        .flex_wrap("wrap")
        .align_items("center")
        .gap(Spacing::SM)
        .property("margin-left", "auto")
        .build();

    rsx! {
        div {
            role: props.role,
            aria_live: props.aria_live,
            class: "notice-bar",
            style: outer_style,
            span { style: label_style, "{props.label}" }
            if let Some(children) = props.children {
                div { style: body_style, {children} }
            }
            if let Some(trailing) = props.trailing {
                div { style: trailing_style, {trailing} }
            }
        }
    }
}
