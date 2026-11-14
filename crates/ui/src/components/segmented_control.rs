// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared segmented button group.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates empty helper enums"
)]

use crate::theme::{ColorScheme, Interaction, Shadow, StyleBuilder, Typography};
use dioxus::prelude::*;

/// Item rendered inside a segmented control.
#[allow(clippy::module_name_repetitions)] // UI component type intentionally matches module name
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentedControlItem {
    /// The display label for this item.
    pub label: &'static str,
    /// The value used for comparison with `selected_value`.
    pub value: &'static str,
}

/// Properties for the [`SegmentedControl`] component.
#[allow(clippy::module_name_repetitions)] // UI component type intentionally matches module name
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
    /// Use dark theme colors.
    #[props(default = false)]
    pub dark: bool,
    /// Items should stretch to fill available space.
    #[props(default = false)]
    pub stretch: bool,
    /// Wrap items to multiple lines if needed.
    #[props(default = true)]
    pub wrap: bool,
    /// ARIA current attribute value when item is active.
    #[props(default = "true")]
    pub active_aria_current: &'static str,
}

/// A segmented button control for selecting one option from multiple.
#[component]
pub fn SegmentedControl(props: SegmentedControlProps) -> Element {
    let SegmentedControlProps {
        aria_label,
        selected_value,
        items,
        on_select,
        dark,
        stretch,
        wrap,
        active_aria_current,
    } = props;

    let group_style = StyleBuilder::new()
        .display("inline-flex")
        .flex_wrap(if wrap { "wrap" } else { "nowrap" })
        .align_items("center")
        .gap("6px")
        .build();

    rsx! {
        div {
            role: "group",
            aria_label,
            style: group_style,
            for item in items {
                SegmentedButton {
                    key: "{item.value}",
                    label: item.label,
                    value: item.value,
                    active: item.value == selected_value,
                    dark,
                    stretch,
                    active_aria_current,
                    on_select,
                }
            }
        }
    }
}

#[derive(Clone, Props, Debug, PartialEq)]
#[allow(clippy::module_name_repetitions)] // UI component type intentionally matches module name
struct SegmentedButtonProps {
    pub label: &'static str,
    pub value: &'static str,
    pub active: bool,
    pub on_select: EventHandler<String>,
    #[props(default = false)]
    pub dark: bool,
    #[props(default = false)]
    pub stretch: bool,
    #[props(default = "true")]
    pub active_aria_current: &'static str,
}

#[component]
fn SegmentedButton(props: SegmentedButtonProps) -> Element {
    let SegmentedButtonProps {
        label,
        value,
        active,
        on_select,
        dark,
        stretch,
        active_aria_current,
    } = props;

    let colors = if dark {
        ColorScheme::DARK
    } else {
        ColorScheme::LIGHT
    };
    let (bg, fg, border_color, shadow) = if active {
        (colors.accent, colors.surface, colors.accent, Shadow::XS)
    } else {
        (colors.surface2, colors.text, colors.border, "none")
    };

    let button_style = StyleBuilder::new()
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
        .flex(if stretch { "1 1 0%" } else { "0 0 auto" })
        .property("white-space", "nowrap")
        .build();

    rsx! {
        button {
            r#type: "button",
            aria_pressed: if active { "true" } else { "false" },
            aria_current: if active { active_aria_current } else { "false" },
            style: button_style,
            onclick: move |_| on_select.call(value.to_string()),
            "{label}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::render_debug;

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
                dark: true,
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
            light.contains("Unit selector"),
            "missing aria_label in {light}"
        );
        assert!(light.contains("PPM"), "missing first item in {light}");
        assert!(light.contains("Da"), "missing second item in {light}");
        assert!(
            light.contains("flex-wrap: wrap"),
            "default segmented control should wrap in {light}"
        );

        let dark = render_debug(render_dark_stretched_nowrap);
        assert!(
            dark.contains("flex-wrap: nowrap"),
            "nowrap prop should apply flex-wrap: nowrap in {dark}"
        );
        assert!(
            dark.contains("flex: 1 1 0%"),
            "stretch prop should flex items equally in {dark}"
        );
        assert!(
            dark.contains("page"),
            "active item should emit custom aria_current in {dark}"
        );
        assert!(
            dark.contains(ColorScheme::DARK.accent),
            "dark active item should use dark accent in {dark}"
        );
    }
}
