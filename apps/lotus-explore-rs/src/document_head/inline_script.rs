// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Externalized JavaScript bridge files compiled into head scripts.

const BOOTSTRAP_INLINE_SCRIPT: &str = include_str!("../../public/assets/js/bootstrap.js");
const RDKIT_BRIDGE_SCRIPT: &str = include_str!("../../public/assets/js/curation/rdkit-bridge.js");
const CITATION_BRIDGE_SCRIPT: &str = include_str!("../../public/assets/js/curation/citation-bridge.js");

/// Primary bootstrap script passed to `DocumentHead { inline_script }`.
pub fn build_bootstrap_inline_script() -> String {
    BOOTSTRAP_INLINE_SCRIPT.to_string()
}

/// Lazy-loaded bridge code for RDKit and Citation.js on curation routes.
pub fn build_curation_inline_script() -> String {
    format!("{RDKIT_BRIDGE_SCRIPT}\n\n{CITATION_BRIDGE_SCRIPT}")
}
