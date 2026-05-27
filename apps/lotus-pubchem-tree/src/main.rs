// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

#![allow(non_snake_case)]

mod api;
mod app;
mod app_state;
mod components;
mod models;
mod services;

use dioxus::prelude::*;

#[cfg(test)]
mod tests;

fn main() {
    let level = if cfg!(debug_assertions) {
        log::Level::Debug
    } else {
        log::Level::Info
    };
    console_log::init_with_level(level).ok();
    launch(app::shell::AppRoot);
}
