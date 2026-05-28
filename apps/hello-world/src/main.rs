//! Minimal Dioxus web-app template.
//!
//! Copy this app to get started:
//!   cp -r apps/hello-world apps/my-new-app
//! Then edit Cargo.toml and Dioxus.toml (change `name`), and add
//! `"apps/my-new-app"` to the workspace members in the root Cargo.toml.

// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use dioxus::prelude::*;

fn main() {
    console_log::init_with_level(log::Level::Info).ok();
    launch(app);
}

#[component]
fn app() -> Element {
    let mut count = use_signal(|| 0i32);

    rsx! {
        div { class: "card",
            h1 { "👋 Hello, Dioxus!" }
            p { "A minimal template — copy this app to build your own." }
            div { class: "count", "{count}" }
            div { class: "row",
                button {
                    class: "btn btn-primary",
                    onclick: move |_| *count.write() += 1,
                    "+ 1"
                }
                button { class: "btn", onclick: move |_| *count.write() -= 1, "− 1" }
                button { class: "btn", onclick: move |_| *count.write() = 0, "Reset" }
            }
        }
    }
}
