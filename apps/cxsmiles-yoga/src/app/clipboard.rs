// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Clipboard button component and wasm32 async copy helpers.

use std::sync::Arc;

use dioxus::prelude::*;
use ui::{ColorScheme, Radius, StyleBuilder, Typography};

#[cfg(target_arch = "wasm32")]
fn copy_to_clipboard(text: &str) -> bool {
    if let Some(win) = web_sys::window() {
        let cb = win.navigator().clipboard();
        drop(cb.write_text(text));
        return true;
    }
    false
}

#[cfg(not(target_arch = "wasm32"))]
const fn copy_to_clipboard(_text: &str) -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
fn trigger_copy(text: &str, mut copied: Signal<bool>) {
    let ok = copy_to_clipboard(text);
    copied.set(ok);
    if ok {
        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(1200).await;
            copied.set(false);
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn trigger_copy(text: &str, mut copied: Signal<bool>) {
    copied.set(copy_to_clipboard(text));
}

#[derive(Props, Clone, PartialEq, Debug)]
pub(super) struct CopyCellProps {
    pub(super) text: Arc<str>,
}

/// Small "Copy" button that flashes "Copied!" on wasm32.
#[component]
pub(super) fn CopyCell(props: CopyCellProps) -> Element {
    let copied = use_signal(|| false);
    let text = props.text;
    let light = ColorScheme::LIGHT;
    let btn_style = StyleBuilder::new()
        .background_color(light.bg)
        .color(light.text2)
        .border(&format!("1px solid {}", light.border))
        .border_radius(Radius::MD)
        .padding("2px 0.7rem")
        .font_size(Typography::UI)
        .font_weight("600")
        .cursor("pointer")
        .build();
    rsx! {
        button {
            r#type: "button",
            onclick: move |_| trigger_copy(&text, copied),
            style: btn_style,
            if *copied.read() {
                "Copied!"
            } else {
                "Copy"
            }
        }
    }
}
