//! Shared segmented button group.

use crate::theme::{ColorScheme, Shadow, StyleBuilder, Typography};
use dioxus::prelude::*;

/// Item rendered inside a segmented control.
#[derive(Clone, Debug, PartialEq)]
pub struct SegmentedControlItem {
    pub label: String,
    pub value: String,
}

/// Properties for the [`SegmentedControl`] component.
#[derive(Clone, Props, Debug, PartialEq)]
pub struct SegmentedControlProps {
    pub aria_label: String,
    pub selected_value: String,
    pub items: Vec<SegmentedControlItem>,
    pub on_select: EventHandler<String>,
    #[props(default = false)]
    pub dark: bool,
    #[props(default = false)]
    pub stretch: bool,
    #[props(default = true)]
    pub wrap: bool,
    #[props(default = "true")]
    pub active_aria_current: &'static str,
}

#[component]
pub fn SegmentedControl(props: SegmentedControlProps) -> Element {
    let colors = if props.dark {
        ColorScheme::DARK
    } else {
        ColorScheme::LIGHT
    };
    let selected_value = props.selected_value.clone();
    let stretch = props.stretch;
    let wrap = props.wrap;
    let on_select = props.on_select;

    let mut group_style = StyleBuilder::new()
        .display("flex")
        .align_items("center")
        .gap("4px")
        .padding("4px")
        .border(&format!("1px solid {}", colors.border))
        .border_radius("999px")
        .background_color(colors.surface)
        .box_shadow(Shadow::XS);
    group_style = if wrap {
        group_style.flex_wrap("wrap")
    } else {
        group_style
            .flex_wrap("nowrap")
            .property("overflow-x", "auto")
    };
    let group_style = group_style.build();

    let button_base_style = StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .padding("6px 12px")
        .min_height("40px")
        .border_radius("999px")
        .font_size(Typography::UI)
        .font_weight("600")
        .property("line-height", "1.2")
        .property("border", "1px solid transparent")
        .property("cursor", "pointer")
        .transition(crate::theme::Interaction::TRANSITION_FAST)
        .build();

    rsx! {
        div { role: "group", aria_label: props.aria_label, style: group_style,
            for item in &props.items {
                SegmentedButton {
                    label: item.label.clone(),
                    value: item.value.clone(),
                    selected_value: selected_value.clone(),
                    dark: props.dark,
                    stretch,
                    active_aria_current: props.active_aria_current,
                   on_select,
                    base_style: button_base_style.clone(),
                }
            }
        }
    }
}

#[derive(Clone, Props, Debug, PartialEq)]
struct SegmentedButtonProps {
    pub label: String,
    pub value: String,
    pub selected_value: String,
    pub on_select: EventHandler<String>,
    #[props(default = false)]
    pub dark: bool,
    #[props(default = false)]
    pub stretch: bool,
    #[props(default = "true")]
    pub active_aria_current: &'static str,
    pub base_style: String,
}

#[component]
fn SegmentedButton(props: SegmentedButtonProps) -> Element {
    let colors = if props.dark {
        ColorScheme::DARK
    } else {
        ColorScheme::LIGHT
    };
    let active = props.value == props.selected_value;
    let style = segmented_button_style(&colors, active, props.stretch, &props.base_style);
    let value = props.value.clone();
    let on_select = props.on_select;

    rsx! {
        button {
            r#type: "button",
            aria_pressed: if active { "true" } else { "false" },
            aria_current: if active {
                props.active_aria_current
            } else {
                "false"
            },
            style: style,
            onclick: move |_| on_select.call(value.clone()),
            "{props.label}"
        }
    }
}

fn segmented_button_style(
    colors: &ColorScheme,
    active: bool,
    stretch: bool,
    base_style: &str,
) -> String {
    let mut style =
        StyleBuilder::new().property("flex", if stretch { "1 1 180px" } else { "0 0 auto" });
    if active {
        style = style
            .background_color(colors.accent)
            .color(colors.bg)
            .border(&format!("1px solid {}", colors.accent));
    } else {
        style = style
            .background_color(colors.surface2)
            .color(colors.text2)
            .border(&format!("1px solid {}", colors.border));
    }
    format!("{}; {}", base_style, style.build())
}
