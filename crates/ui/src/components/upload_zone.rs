// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Shared file-drop zone (drag + browse), de-duplicating the per-app upload
//! pattern found in `apps/json-count-rs` and `apps/lipid-selecto-rs`.
//!
//! Provides the shared rendering (Card + hidden `<input>`), the drag‑state
//! toggling and the `file_name`/`status`/`busy` bookkeeping. The app owns the
//! signals (passed by value — `Signal` is `Copy`) and supplies the app‑specific
//! file processing via `EventHandler` callbacks.
//!
//! Why `EventHandler` props (not a bundled handlers struct): a generic
//! `struct UploadHandlers<…>` of bare closures only compiles when the call site
//! mono‑morphises it in the same crate; shared components in `crates/ui` have no
//! same‑crate call site, so the closures fail the rsx `on*` coercion.
//! `EventHandler` props are wired here with bare `move |e| … .call(e)` closures
//! (`ListenerCallback`), never a stored `Callback` on an rsx prop.
//!
//! The blob extraction (`upload::extract_blob_from_file_data`) is intentionally
//! kept in the apps — so `crates/ui` stays free of the `crates/upload` dep.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus component macro generates a Props helper empty enum"
)]

use crate::theme::{ColorScheme, Radius, Spacing, StyleBuilder, Typography};
use dioxus::events::{DragData, FormData};
use dioxus::prelude::*;

#[component]
pub fn UploadZone(
    /// Filename of the selected file (displayed under the drop zone).
    file_name: Signal<String>,
    /// Human-readable status ("Choose a JSON file…", "Scanning…", error msg).
    status: Signal<String>,
    /// Whether a scan/analysis is in flight (disables the input).
    busy: Signal<bool>,
    /// Whether a drag is currently over the zone (toggles the dashed border).
    drag_active: Signal<bool>,
    /// Browse or programmatic file pick — app extracts the blob + processes it.
    on_file_change: EventHandler<Event<FormData>>,
    /// Drop — app extracts the blob + processes it (the shared handler already
    /// resets `drag_active` before delegating).
    on_drop: EventHandler<Event<DragData>>,
    /// `<input type=file>` accept attribute.
    accept: &'static str,
    /// Prominent label inside the drop zone.
    label: &'static str,
    /// Small hint under the label.
    hint: &'static str,
    /// Leading icon/glyph for the label.
    icon: &'static str,
) -> Element {
    let colors = ColorScheme::LIGHT;
    let mut drag_active = drag_active;
    let on_file_change = on_file_change;
    let on_drop = on_drop;

    let is_drag_active = *drag_active.read();
    let upload_style = StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .align_items("center")
        .justify_content("center")
        .gap(Spacing::MD)
        .property("min-height", "140px")
        .width("100%")
        .property("box-sizing", "border-box")
        .property("position", "relative")
        .border(&format!(
            "2px dashed {}",
            if is_drag_active {
                colors.blue
            } else {
                colors.border
            }
        ))
        .border_radius(Radius::MD)
        .padding(Spacing::LG)
        .cursor("pointer")
        .background_color(if is_drag_active {
            colors.surface2
        } else {
            colors.surface
        })
        .color(colors.text2)
        .build();

    let status_text = status.read();
    let status_color = if status_text.contains("Error")
        || status_text.contains("error")
        || status_text.contains("Unexpected")
    {
        colors.red
    } else {
        colors.text
    };

    rsx! {
        label {
            r#for: "upload-input",
            style: upload_style,
            ondragenter: move |evt: Event<DragData>| {
                evt.prevent_default();
                drag_active.set(true);
            },
            ondragover: move |evt: Event<DragData>| {
                evt.prevent_default();
                drag_active.set(true);
            },
            ondragleave: move |evt: Event<DragData>| {
                evt.prevent_default();
                drag_active.set(false);
            },
            ondrop: move |evt: Event<DragData>| {
                evt.prevent_default();
                drag_active.set(false);
                on_drop.call(evt);
            },

            span {
                style: StyleBuilder::new()
                    .font_size(Typography::BODY)
                    .font_weight("600")
                    .build(),
                "{icon} {label}"
            }
            span {
                style: StyleBuilder::new()
                    .font_size(Typography::LABEL)
                    .font_weight("500")
                    .color(colors.text3)
                    .build(),
                "{hint}"
            }

            input {
                id: "upload-input",
                r#type: "file",
                accept: accept,
                disabled: *busy.read(),
                onchange: move |evt: Event<FormData>| on_file_change.call(evt),
                aria_describedby: "upload-status",
                style: StyleBuilder::new()
                    .property("position", "absolute")
                    .property("inset", "0")
                    .width("100%")
                    .height("100%")
                    .opacity("0")
                    .cursor("pointer")
                    .build(),
            }
        }

        if !file_name.read().is_empty() {
            p {
                style: StyleBuilder::new()
                    .margin(&format!("{} 0 0", Spacing::MD))
                    .color(colors.text2)
                    .font_size(Typography::BODY)
                    .build(),
                "Selected: {file_name.read()}"
            }
        }

        p {
            id: "upload-status",
            role: "status",
            aria_live: "polite",
            aria_atomic: "true",
            style: StyleBuilder::new()
                .margin(&format!("{} 0 0", Spacing::MD))
                .font_weight("600")
                .color(status_color)
                .build(),
            "{status.read()}"
        }
    }
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
            }
        }
    }

    #[test]
    fn upload_zone_renders_idle_and_drag_error_states() {
        let idle = render_debug(render_idle_upload_zone);
        assert!(
            idle.contains("[MGF] Drop MGF file"),
            "missing icon/label in {idle}"
        );
        assert!(idle.contains(".mgf"), "missing accept attribute in {idle}");
        assert!(
            !idle.contains("Selected:"),
            "idle upload zone should hide selected filename in {idle}"
        );

        let active = render_debug(render_active_error_upload_zone);
        assert!(
            active.contains("Selected: spectra.mgf"),
            "populated upload zone should show filename in {active}"
        );
        assert!(
            active.contains("Error parsing header"),
            "missing error status message in {active}"
        );
        assert!(
            active.contains(ColorScheme::LIGHT.red),
            "error status should color text red in {active}"
        );
        assert!(
            active.contains(ColorScheme::LIGHT.blue),
            "active drag should highlight border blue in {active}"
        );
    }
}
