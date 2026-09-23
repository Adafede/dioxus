// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared segmented button group.

use dioxus::prelude::*;

/// Item rendered inside a segmented control.
#[allow(clippy::module_name_repetitions)] // UI component type intentionally matches module name
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SegmentedControlItem {
    /// The display label for this item.
    pub label: String,
    /// The value used for comparison with `selected_value`.
    pub value: String,
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
    pub items: Vec<SegmentedControlItem>,
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
    let selected_value = props.selected_value.clone();
    let stretch = props.stretch;
    let wrap = props.wrap;
    let on_select = props.on_select;

    let group_classes = if wrap {
        "inline-flex flex-wrap items-center gap-1 shrink-0"
    } else {
        "inline-flex items-center gap-1 shrink-0"
    };

    rsx! {
        div {
            role: "group",
            aria_label: props.aria_label,
            class: "{group_classes}",
            for item in &props.items {
                SegmentedButton {
                    label: item.label.clone(),
                    value: item.value.clone(),
                    selected_value: selected_value.clone(),
                    dark: props.dark,
                    stretch,
                    active_aria_current: props.active_aria_current,
                    on_select,
                }
            }
        }
    }
}

#[derive(Clone, Props, Debug, PartialEq)]
#[allow(clippy::module_name_repetitions)] // UI component type intentionally matches module name
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
}

#[component]
fn SegmentedButton(props: SegmentedButtonProps) -> Element {
    let active = props.value == props.selected_value;

    let stretch = props.stretch;
    let on_select = props.on_select;
    let value = props.value.clone();
    let label = props.label.clone();

    let active_classes = if active {
        "bg-accent text-bg border-accent shadow-xs"
    } else if props.dark {
        "bg-surface2 text-text2 border-border"
    } else {
        "bg-surface text-text border-border"
    };

    let flex_class = if stretch {
        "flex-1 min-w-0"
    } else {
        "flex-none"
    };

    let classes = format!(
        "inline-flex items-center justify-center px-5 py-1.5 text-ui leading-none font-semibold rounded-full border transition-transform duration-150 active:scale-[0.98] min-h-[40px] whitespace-nowrap cursor-pointer focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2 {flex_class} {active_classes}"
    );

    rsx! {
        button {
            r#type: "button",
            aria_pressed: if active { "true" } else { "false" },
            aria_current: if active { props.active_aria_current } else { "false" },
            class: "{classes}",
            onclick: move |_| on_select.call(value.clone()),
            "{label}"
        }
    }
}
