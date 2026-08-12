//! Shared notice bar component.

use crate::theme::{ColorScheme, Radius, Shadow, Spacing, StyleBuilder, Typography};
use dioxus::prelude::*;

/// Visual tone for a notice bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeTone {
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
}

/// Properties for the [`NoticeBar`] component.
#[derive(Clone, Props, Debug, PartialEq)]
pub struct NoticeBarProps {
    pub label: String,
    #[props(default = NoticeTone::Neutral)]
    pub tone: NoticeTone,
    #[props(default = "status")]
    pub role: &'static str,
    #[props(default = "polite")]
    pub aria_live: &'static str,
    #[props(default = false)]
    pub dark: bool,
    #[props(default = "10px 22px 0")]
    pub margin: &'static str,
    #[props(default = "9px 12px")]
    pub padding: &'static str,
    #[props(default)]
    pub trailing: Option<Element>,
    #[props(default)]
    pub children: Option<Element>,
}

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
        .border(&format!("1px solid {}", border_color))
        .border_left(&format!("4px solid {}", tone_color))
        .border_radius(Radius::MD)
        .background_color(&outer_background)
        .box_shadow(Shadow::XS)
        .font_size(Typography::UI)
        .transition(crate::theme::Interaction::TRANSITION_FAST)
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
        div { role: props.role, aria_live: props.aria_live, style: outer_style,
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
