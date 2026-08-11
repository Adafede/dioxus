//! `mgf-precursor-erro-rs` — MGF precursor mass-error analysis.
//!
//! Uploads an MGF file, recalibrates precursor *m/z* values, and visualises
//! the resulting mass-error distribution as an interactive histogram.

fn main() {
    dioxus::launch(mgf_precursor_erro_rs::app);
}
