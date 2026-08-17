//! `cxsmiles-yoga` — generate CX-SMILES from lists of related structures.
//!
//! Core algorithm lives in [`cxsmiles`] (a pure-Rust, UI-free module that can
//! be unit-tested and reused outside the Dioxus tree — see `cargo test -p
//! cxsmiles-yoga --lib`).  The Dioxus `app` entry point ([`app::app`]) is a
//! thin shell: a textarea, a "generate" button, and a results panel that shows
//! the produced CX-SMILES, the detected construct type(s), a round-trip
//! confidence indicator, and 2D depiction of the core scaffold.
//!
//! ## Pipeline
//!
//! 1. **Parse** the input SMILES list (one per line).
//! 2. **Cluster** candidates by ECFP4/Tanimoto similarity so unrelated
//!    structures are never forced into one nonsensical CX-SMILES.
//! 3. **Maximum common substructure** across each cluster (via `chematic`'s
//!    `McGregor` search).
//! 4. **Diff** each member against the MCS to isolate variable fragments and
//!    classify them as positional-isomer (`m:`) or variable-length repeat
//!    (`Sg:n:`) construct.
//! 5. **Serialize** the core scaffold + CX extension fields.
//! 6. **Round-trip**: expand the CX-SMILES back to concrete SMILES and check it
//!    re-covers the inputs; the coverage becomes the confidence indicator.
//!
//! ## Usage
//!
//! ### Locally
//!
//! ```bash
//! dx serve --package cxsmiles-yoga
//! ```
//!
//! ### Build for deployment
//!
//! ```bash
//! dx build --release --platform web --package cxsmiles-yoga
//! ```
//!
//! ### Tests
//!
//! ```bash
//! cargo test --lib -p cxsmiles-yoga
//! ```

pub mod app;
pub mod cxsmiles;
pub mod depict;
pub mod examples;

pub use app::app;
