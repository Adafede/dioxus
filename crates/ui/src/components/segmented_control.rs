//! Shared segmented button group.

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
        "inline-flex items-center justify-center px-5 py-1.5 text-ui leading-none font-semibold rounded-full border transition-transform duration-150 active:scale-[0.98] min-h-[40px] whitespace-nowrap cursor-pointer focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-accent/28 focus-visible:ring-offset-2 {} {}",
        flex_class, active_classes
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
