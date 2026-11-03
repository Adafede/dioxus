// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Programmatic document `<head>` management for smellfish-rs.
//!
//! Sets meta tags, loads CDN scripts, the local motif-library bridge, and the
//! stylesheet from [`crate::styles::CSS`].
//!
//! The RDKit/NP-likeness bridge JavaScript lives in [`crate::rdkit_bridge`].

use crate::rdkit_bridge::RDKIT_BRIDGE_JS;
use crate::styles::CSS;
use dioxus::prelude::*;
use ui::prelude::*;

/// Renders the document `<head>` for smellfish-rs.
#[component]
pub fn SmellfishDocumentHead() -> Element {
    let description = "Drop a CSV of SMILES, render molecules with RDKit.js, and score \
        natural-product originality with Ertl-style NP-likeness, LOTUS/PubChem evidence, \
        and dataset-derived scaffold and decoration motifs.";

    let scripts = vec![
        "https://unpkg.com/@rdkit/rdkit/dist/RDKit_minimal.js".to_string(),
        "https://scripts.simpleanalyticscdn.com/latest.js".to_string(),
    ];

    rsx! {
        DocumentHead {
            title: "smellfish-rs".to_string(),
            webmcp: Some(WebMcpConfig {
                app_id: "smellfish-rs",
                title: "Smellfish-rs",
                description: "Smellfish-rs literature-backed NP-likeness scoring with RDKit.js, QLever enrichment, and explicit endpoint health checks",
                inputs: &["smiles_csv"],
                outputs: &["np_likeness", "motifs", "enrichment"],
                edam: EdamAnnotations {
                    topics: &[
                        EdamTerm { id: "topic_0154", label: "Small molecules" },
                        EdamTerm { id: "topic_2258", label: "Cheminformatics" },
                        EdamTerm { id: "topic_3172", label: "Metabolomics" },
                    ],
                    operations: &[
                        EdamTerm { id: "operation_0250", label: "Molecular property calculation" },
                        EdamTerm { id: "operation_2421", label: "Database search" },
                        EdamTerm { id: "operation_0570", label: "Structure visualisation" },
                    ],
                    input_data: &[EdamTerm { id: "data_2301", label: "SMILES string" }],
                    input_formats: &[
                        EdamTerm { id: "format_3752", label: "CSV" },
                        EdamTerm { id: "format_1196", label: "SMILES" },
                    ],
                    output_data: &[EdamTerm { id: "data_2048", label: "Report" }],
                    output_formats: &[EdamTerm { id: "format_3752", label: "CSV" }],
                },
            }),
            lang: "en".to_string(),
            description: Some(description.to_string()),
            theme_colors: Some(("#f6f8fb", "#10141b")),
            scripts,
            inline_script: Some(RDKIT_BRIDGE_JS.to_string()),
        }

        // Keep the local LOTUS-inspired styling in the initial HTML response so
        // the UI is not blank during the first WASM load.
        document::Style { "{CSS}" }

        // Local motif-library script (must load before inline bridge JS)
        document::Script { src: "motif-library.js" }

        // Resource hints — preconnect for external origins
        document::Link { rel: "preconnect", href: "https://unpkg.com", crossorigin: "anonymous" }
        document::Link { rel: "preconnect", href: "https://qlever.dev", crossorigin: "anonymous" }
        document::Link { rel: "preconnect", href: "https://qlever.cs.uni-freiburg.de", crossorigin: "anonymous" }
    }
}
