// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared segmented button group.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates empty helper enums"
)]

use crate::theme::{Interaction, Shadow, StyleBuilder, Theme, Typography};
use dioxus::prelude::*;

/// Item rendered inside a [`SegmentedControl`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentedControlItem {
    /// Display label for this option.
    pub label: &'static str,
    /// Value compared against `selected_value`.
    pub value: &'static str,
}

/// Properties for the [`SegmentedControl`] component.
#[derive(Clone, Props, Debug, PartialEq)]
pub struct SegmentedControlProps {
    /// ARIA label for the control group.
    pub aria_label: String,
    /// Currently selected value.
    pub selected_value: String,
    /// Items to render in the control.
    pub items: &'static [SegmentedControlItem],
    /// Callback fired when selection changes.
    pub on_select: EventHandler<String>,
    /// Light/dark color theme.
    #[props(default)]
    pub theme: Theme,
    /// Stretch items evenly across available horizontal space.
    #[props(default = false)]
    pub stretch: bool,
    /// Allow wrapping items across lines.
    #[props(default = true)]
    pub wrap: bool,
    /// ARIA current attribute value when item is active.
    #[props(default = "true")]
    pub active_aria_current: &'static str,
}

/// Segmented button control for selecting one option from a static list.
#[component]
pub fn SegmentedControl(props: SegmentedControlProps) -> Element {
    let group_style = StyleBuilder::with_capacity(96)
        .display("inline-flex")
        .flex_wrap(if props.wrap { "wrap" } else { "nowrap" })
        .align_items("center")
        .gap("6px")
        .build();

    rsx! {
        div {
            role: "group",
            aria_label: props.aria_label,
            style: group_style,
            for item in props.items {
                SegmentedButton {
                    key: "{item.value}",
                    label: item.label,
                    value: item.value,
                    active: item.value == props.selected_value,
                    theme: props.theme,
                    stretch: props.stretch,
                    active_aria_current: props.active_aria_current,
                    on_select: props.on_select,
                }
            }
        }
    }
}

#[derive(Clone, Props, Debug, PartialEq)]
struct SegmentedButtonProps {
    pub label: &'static str,
    pub value: &'static str,
    pub active: bool,
    pub on_select: EventHandler<String>,
    #[props(default)]
    pub theme: Theme,
    #[props(default = false)]
    pub stretch: bool,
    #[props(default = "true")]
    pub active_aria_current: &'static str,
}

#[component]
fn SegmentedButton(props: SegmentedButtonProps) -> Element {
    let colors = props.theme.colors();
    let (bg, fg, border_color, shadow) = if props.active {
        (colors.accent, colors.surface, colors.accent, Shadow::XS)
    } else {
        (colors.surface2, colors.text, colors.border, "none")
    };

    let button_style = StyleBuilder::with_capacity(320)
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .padding("6px 16px")
        .min_height("38px")
        .font_size(Typography::UI)
        .font_family(Typography::SANS)
        .font_weight("600")
        .line_height("1")
        .border_radius("999px")
        .background_color(bg)
        .color(fg)
        .border(&format!("1px solid {border_color}"))
        .box_shadow(shadow)
        .cursor("pointer")
        .transition(Interaction::TRANSITION_DEFAULT)
        .flex(if props.stretch { "1 1 0%" } else { "0 0 auto" })
        .property("white-space", "nowrap")
        .build();

    let value = props.value;
    let on_select = props.on_select;
    rsx! {
        button {
            r#type: "button",
            aria_pressed: if props.active { "true" } else { "false" },
            aria_current: if props.active { props.active_aria_current } else { "false" },
            style: button_style,
            onclick: move |_| on_select.call(value.to_string()),
            "{props.label}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::render_debug;
    use crate::theme::ColorScheme;

    static ITEMS: &[SegmentedControlItem] = &[
        SegmentedControlItem {
            label: "PPM",
            value: "ppm",
        },
        SegmentedControlItem {
            label: "Da",
            value: "da",
        },
    ];

    fn render_light_control() -> Element {
        rsx! {
            SegmentedControl {
                aria_label: "Unit selector".to_string(),
                selected_value: "ppm".to_string(),
                items: ITEMS,
                on_select: |_| {},
            }
        }
    }

    fn render_dark_stretched_nowrap() -> Element {
        rsx! {
            SegmentedControl {
                aria_label: "Dark selector".to_string(),
                selected_value: "da".to_string(),
                items: ITEMS,
                theme: Theme::Dark,
                stretch: true,
                wrap: false,
                active_aria_current: "page",
                on_select: |_| {},
            }
        }
    }

    #[test]
    fn segmented_control_renders_active_state_and_layout_options() {
        let light = render_debug(render_light_control);
        assert!(
            light.contains("Unit selector")
                && light.contains("PPM")
                && light.contains("Da")
                && light.contains("flex-wrap: wrap"),
            "light segmented control mismatch: {light}"
        );

        let dark = render_debug(render_dark_stretched_nowrap);
        assert!(
            dark.contains("flex-wrap: nowrap")
                && dark.contains("flex: 1 1 0%")
                && dark.contains("page")
                && dark.contains(ColorScheme::DARK.accent),
            "dark segmented control mismatch: {dark}"
        );
    }
}
