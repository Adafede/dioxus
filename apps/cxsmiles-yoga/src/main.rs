//! `cxsmiles-yoga` — generate CX-SMILES from lists of related structures.
//!
//! Launch entry point; the component tree lives in the `lib` crate so that the
//! same `app` works for both native dev (`cargo run -p cxsmiles-yoga`) and WASM
//! (`dx serve --package cxsmiles-yoga`), mirroring `lipid-selecto-rs`.

fn main() {
    dioxus::launch(cxsmiles_yoga::app);
}
