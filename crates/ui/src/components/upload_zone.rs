// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared file-drop zone (drag + browse) for local browser uploads.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates a Props helper empty enum"
)]

use crate::theme::{ColorScheme, Radius, Spacing, StyleBuilder, Theme, Typography};
use dioxus::events::{DragData, FormData};
use dioxus::prelude::*;

fn status_color(message: &str, colors: ColorScheme) -> &'static str {
    if message.contains("Error") || message.contains("error") || message.contains("Unexpected") {
        colors.red
    } else {
        colors.text
    }
}

/// Properties for the [`UploadZone`] component.
#[derive(Clone, Props, PartialEq)]
pub struct UploadZoneProps {
    /// Filename of the selected file.
    pub file_name: Signal<String>,
    /// Human-readable status text.
    pub status: Signal<String>,
    /// Whether an import/scan is currently running.
    pub busy: Signal<bool>,
    /// Whether a drag operation is hovering over the zone.
    pub drag_active: Signal<bool>,
    /// Browse or programmatic file change handler.
    pub on_file_change: EventHandler<Event<FormData>>,
    /// Drag-and-drop handler.
    pub on_drop: EventHandler<Event<DragData>>,
    /// `<input type=file>` accept filter.
    pub accept: &'static str,
    /// Prominent action label.
    pub label: &'static str,
    /// Secondary hint text.
    pub hint: &'static str,
    /// Leading glyph / badge.
    pub icon: &'static str,
    /// Unique DOM ID prefix preventing collisions when multiple zones appear on one page.
    #[props(default = "upload")]
    pub id_prefix: &'static str,
    /// Light/dark color theme.
    #[props(default)]
    pub theme: Theme,
}

impl core::fmt::Debug for UploadZoneProps {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UploadZoneProps")
            .field("accept", &self.accept)
            .field("label", &self.label)
            .field("id_prefix", &self.id_prefix)
            .field("theme", &self.theme)
            .finish_non_exhaustive()
    }
}

/// Accessible drag-and-drop + file picker zone.
#[component]
pub fn UploadZone(props: UploadZoneProps) -> Element {
    let colors = props.theme.colors();
    let mut drag_active = props.drag_active;
    let is_drag_active = *drag_active.read();
    let status_text = props.status.read();
    let tone_color = status_color(&status_text, colors);
    let input_id = format!("{}-input", props.id_prefix);
    let status_id = format!("{}-status", props.id_prefix);

    let zone_style = build_zone_style(colors, is_drag_active);
    let hidden_input_style = build_hidden_input_style();
    let activate_drag = move |evt: Event<DragData>| {
        evt.prevent_default();
        drag_active.set(true);
    };

    rsx! {
        label {
            r#for: "{input_id}",
            style: zone_style,
            ondragenter: activate_drag,
            ondragover: activate_drag,
            ondragleave: move |evt: Event<DragData>| {
                evt.prevent_default();
                drag_active.set(false);
            },
            ondrop: move |evt: Event<DragData>| {
                evt.prevent_default();
                drag_active.set(false);
                props.on_drop.call(evt);
            },
            span {
                style: StyleBuilder::with_capacity(48).font_size(Typography::BODY).font_weight("600").build(),
                "{props.icon} {props.label}"
            }
            span {
                style: StyleBuilder::with_capacity(80).font_size(Typography::LABEL).font_weight("500").color(colors.text3).build(),
                "{props.hint}"
            }
            input {
                id: "{input_id}",
                r#type: "file",
                accept: props.accept,
                disabled: *props.busy.read(),
                onchange: move |evt: Event<FormData>| props.on_file_change.call(evt),
                aria_describedby: "{status_id}",
                style: hidden_input_style,
            }
        }
        if !props.file_name.read().is_empty() {
            p {
                style: StyleBuilder::with_capacity(96).margin(&format!("{} 0 0", Spacing::MD)).color(colors.text2).font_size(Typography::BODY).build(),
                "Selected: {props.file_name.read()}"
            }
        }
        p {
            id: "{status_id}",
            role: "status",
            aria_live: "polite",
            aria_atomic: "true",
            style: StyleBuilder::with_capacity(96).margin(&format!("{} 0 0", Spacing::MD)).font_weight("600").color(tone_color).build(),
            "{status_text}"
        }
    }
}

fn build_zone_style(colors: ColorScheme, drag_active: bool) -> String {
    StyleBuilder::with_capacity(320)
        .display("flex")
        .flex_direction("column")
        .align_items("center")
        .justify_content("center")
        .gap(Spacing::MD)
        .min_height("140px")
        .width("100%")
        .property("box-sizing", "border-box")
        .property("position", "relative")
        .border(&format!(
            "2px dashed {}",
            if drag_active {
                colors.blue
            } else {
                colors.border
            }
        ))
        .border_radius(Radius::MD)
        .padding(Spacing::LG)
        .cursor("pointer")
        .background_color(if drag_active {
            colors.surface2
        } else {
            colors.surface
        })
        .color(colors.text2)
        .build()
}

fn build_hidden_input_style() -> String {
    StyleBuilder::with_capacity(110)
        .property("position", "absolute")
        .property("inset", "0")
        .width("100%")
        .height("100%")
        .opacity("0")
        .cursor("pointer")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::render_debug;

    fn render_idle_upload_zone() -> Element {
        let file_name = use_signal(String::new);
        let status = use_signal(|| "Ready".to_string());
        let busy = use_signal(|| false);
        let drag_active = use_signal(|| false);
        rsx! {
            UploadZone {
                file_name,
                status,
                busy,
                drag_active,
                on_file_change: |_| {},
                on_drop: |_| {},
                accept: ".mgf",
                label: "Drop MGF file",
                hint: "Client-side only",
                icon: "[MGF]",
            }
        }
    }

    fn render_active_error_upload_zone() -> Element {
        let file_name = use_signal(|| "spectra.mgf".to_string());
        let status = use_signal(|| "Error parsing header".to_string());
        let busy = use_signal(|| true);
        let drag_active = use_signal(|| true);
        rsx! {
            UploadZone {
                file_name,
                status,
                busy,
                drag_active,
                on_file_change: |_| {},
                on_drop: |_| {},
                accept: ".json",
                label: "Drop JSON",
                hint: "Max 500 MB",
                icon: "[JSON]",
                id_prefix: "custom-upload",
            }
        }
    }

    #[test]
    fn upload_zone_renders_idle_and_drag_error_states() {
        let idle = render_debug(render_idle_upload_zone);
        assert!(idle.contains("[MGF] Drop MGF file") && idle.contains(".mgf"));
        assert!(!idle.contains("Selected:"));

        let active = render_debug(render_active_error_upload_zone);
        assert!(
            active.contains("Selected: spectra.mgf")
                && active.contains("Error parsing header")
                && active.contains("custom-upload-input")
                && active.contains("custom-upload-status")
                && active.contains(ColorScheme::LIGHT.red)
                && active.contains(ColorScheme::LIGHT.blue),
            "active upload zone mismatch: {active}"
        );
    }
}
