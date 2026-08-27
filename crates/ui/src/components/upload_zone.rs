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
//! (ListenerCallback), never a stored `Callback` on an rsx prop.
//!
//! The blob extraction (`upload::extract_blob_from_file_data`) is intentionally
//! kept in the apps — so `crates/ui` stays free of the `crates/upload` dep.

use crate::theme::{ColorScheme, Interaction, Radius, Spacing, StyleBuilder, Typography};
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
            if *drag_active.read() {
                colors.blue
            } else {
                colors.border
            }
        ))
        .border_radius(Radius::MD)
        .padding(Spacing::LG)
        .cursor("pointer")
        .background_color(if *drag_active.read() {
            colors.surface2
        } else {
            colors.surface
        })
        .color(colors.text2)
        .build();

    let status_color = if status.read().contains("Error") {
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
