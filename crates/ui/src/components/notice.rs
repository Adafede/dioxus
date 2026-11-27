// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared notice bar component.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates an empty helper enum"
)]

use crate::theme::{Radius, Shadow, Spacing, StyleBuilder, Theme, Typography};
use dioxus::prelude::*;

/// Visual tone for a notice bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeTone {
    /// Neutral informational tone.
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
#[derive(Clone, Props, Debug, PartialEq)]
pub struct NoticeBarProps {
    /// Main label badge displayed in the notice bar.
    pub label: String,
    /// Visual tone/style of the notice bar.
    #[props(default = NoticeTone::Neutral)]
    pub tone: NoticeTone,
    /// ARIA role attribute for accessibility.
    #[props(default = "status")]
    pub role: &'static str,
    /// ARIA live region setting.
    #[props(default = "polite")]
    pub aria_live: &'static str,
    /// Light/dark color theme.
    #[props(default)]
    pub theme: Theme,
    /// CSS margin for the outer container.
    #[props(default = "10px 22px 0")]
    pub margin: &'static str,
    /// CSS padding for the inner content area.
    #[props(default = "9px 12px")]
    pub padding: &'static str,
    /// Optional trailing content.
    #[props(default)]
    pub trailing: Option<Element>,
    /// Optional child body elements.
    #[props(default)]
    pub children: Option<Element>,
}

/// A notice bar component for displaying informational messages with visual tone styling.
#[component]
pub fn NoticeBar(props: NoticeBarProps) -> Element {
    let colors = props.theme.colors();
    let tone_color = match props.tone {
        NoticeTone::Neutral => colors.accent,
        NoticeTone::Info => colors.blue,
        NoticeTone::Success => colors.green,
        NoticeTone::Warning => colors.yellow,
        NoticeTone::Danger => colors.red,
    };
    let outer_style = StyleBuilder::flex_row_wrap(300, Spacing::SM)
        .margin(props.margin)
        .padding(props.padding)
        .property_fmt(
            "border",
            format_args!(
                "1px solid color-mix(in srgb, {tone_color} 24%, {})",
                colors.border
            ),
        )
        .property_fmt("border-left", format_args!("4px solid {tone_color}"))
        .border_radius(Radius::MD)
        .property_fmt(
            "background-color",
            format_args!("color-mix(in srgb, {tone_color} 4%, {})", colors.bg2),
        )
        .box_shadow(Shadow::XS)
        .font_size(Typography::UI)
        .build();

    let label_style = StyleBuilder::with_capacity(240)
        .display("inline-flex")
        .align_items("center")
        .padding("2px 8px")
        .border_radius("999px")
        .property_fmt(
            "background-color",
            format_args!("color-mix(in srgb, {tone_color} 12%, {})", colors.bg2),
        )
        .color(tone_color)
        .font_size(Typography::LABEL)
        .font_weight("700")
        .property("letter-spacing", "0.08em")
        .property("text-transform", "uppercase")
        .property("flex-shrink", "0")
        .property("white-space", "nowrap")
        .build();

    let body_style = StyleBuilder::flex_row_wrap(140, Spacing::SM)
        .property("min-width", "0")
        .property("flex", "1")
        .color(colors.text)
        .build();

    let trailing_style = StyleBuilder::flex_row_wrap(110, Spacing::SM)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::render_debug;
    use crate::theme::ColorScheme;

    fn render_danger_light() -> Element {
        let retry = "Retry";
        let body = "Invalid SMILES line 4";
        rsx! {
            NoticeBar {
                label: "Failed".to_string(),
                tone: NoticeTone::Danger,
                role: "alert",
                aria_live: "assertive",
                trailing: Some(rsx! { span { "{retry}" } }),
                "{body}"
            }
        }
    }

    fn render_all_dark_tones() -> Element {
        rsx! {
            NoticeBar { label: "N".to_string(), tone: NoticeTone::Neutral, theme: Theme::Dark }
            NoticeBar { label: "I".to_string(), tone: NoticeTone::Info, theme: Theme::Dark }
            NoticeBar { label: "S".to_string(), tone: NoticeTone::Success, theme: Theme::Dark }
            NoticeBar { label: "W".to_string(), tone: NoticeTone::Warning, theme: Theme::Dark }
        }
    }

    #[test]
    fn notice_bar_renders_tones_aria_and_slots() {
        let danger = render_debug(render_danger_light);
        assert!(
            danger.contains("Failed")
                && danger.contains("Retry")
                && danger.contains("Invalid SMILES line 4")
                && danger.contains("alert")
                && danger.contains(ColorScheme::LIGHT.red),
            "danger notice mismatch: {danger}"
        );

        let all_dark = render_debug(render_all_dark_tones);
        assert!(
            all_dark.contains(ColorScheme::DARK.blue)
                && all_dark.contains(ColorScheme::DARK.green)
                && all_dark.contains(ColorScheme::DARK.yellow),
            "dark tones mismatch: {all_dark}"
        );
    }
}
