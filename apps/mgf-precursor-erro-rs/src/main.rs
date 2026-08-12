//! `mgf-precursor-erro-rs` — MGF precursor mass-error analysis.
//!
//! Uploads an MGF file, recalibrates precursor *m/z* values, and visualises
//! the resulting mass-error distribution as an interactive histogram.
//!
//! # Run locally
//!
//! ```bash
//! dx serve --package mgf-precursor-erro-rs
//! ```
//!
//! # Build for the website
//!
//! ```bash
//! dx build --release --platform web --package mgf-precursor-erro-rs
//! ```

fn main() {
    dioxus::launch(mgf_precursor_erro_rs::app);
}
