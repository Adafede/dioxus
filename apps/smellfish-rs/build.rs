// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

// A build script has no `Result`-returning `main`: `panic!` (or `assert!`)
// is the only way to fail the build with a diagnostic, which is exactly the
// desired behavior here. Not reachable in the shipped WASM bundle.
#![expect(clippy::panic)]

use std::path::Path;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|err| panic!("CARGO_MANIFEST_DIR is not set: {err}"));
    let manifest_dir = Path::new(&manifest_dir);

    for path in [
        "build.rs",
        "assets/smellfish.css",
        "assets/rdkit_bridge.js",
        "public/ertl_source_vs_synthetic.txt",
        "public/ertl_kingdom_enrichment.txt",
        "public/motif-library.js",
        "public/ertl_npsubstituents.txt",
        "public/lotus_1percent_scaffolds.txt",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }

    for required in [
        "public/ertl_npsubstituents.txt",
        "public/lotus_1percent_scaffolds.txt",
    ] {
        let full = manifest_dir.join(required);
        assert!(
            full.metadata().is_ok_and(|m| m.len() > 0),
            "missing or empty checked-in asset: {}",
            full.display()
        );
    }
}
