mod app;
mod csv;
mod demo;
mod evidence;
mod literature;
mod model;
#[cfg(target_arch = "wasm32")]
mod pipeline;
#[cfg(target_arch = "wasm32")]
mod qlever;
#[cfg(target_arch = "wasm32")]
mod rdkit;
mod styles;

use dioxus::prelude::launch;

fn main() {
    launch(app::app);
}
