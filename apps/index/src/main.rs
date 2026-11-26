// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the index project

//! Landing page for the Dioxus experiments in this workspace.
//!
//! # Run locally
//!
//! ```bash
//! dx serve --package index
//! ```
//!
//! # Build for deployment
//!
//! ```bash
//! dx build --profile wasm-release --platform web --package index
//! ```

use dioxus::prelude::*;
use ui::{
    ColorScheme, DocumentHead, EdamAnnotations, EdamTerm, Footer, Header, Radius,
    SimpleAnalyticsBadge, Spacing, StyleBuilder, Typography, WebMcpConfig, skip_link,
};

mod app_card;

use app_card::{AppCard, AppInfo};

/// Root application component: theme-aware landing page.
#[component]
fn app() -> Element {
    let container_style = StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .property("min-height", "100vh")
        .background_color(ColorScheme::LIGHT.bg)
        .color(ColorScheme::LIGHT.text)
        .build();

    let main_style = StyleBuilder::new()
        .flex("1")
        .padding(&format!("{} {}", Spacing::XL, Spacing::LG))
        .build();

    let grid_style = StyleBuilder::new()
        .display("grid")
        .property(
            "grid-template-columns",
            "repeat(auto-fit, minmax(300px, 1fr))",
        )
        .gap(Spacing::LG)
        .margin(&format!("{} 0", Spacing::XL))
        .build();

    let disclaimer_style = StyleBuilder::new()
        .background_color(ColorScheme::LIGHT.surface)
        .border(&format!("1px solid {}", ColorScheme::LIGHT.border))
        .border_radius(Radius::MD)
        .padding(Spacing::LG)
        .margin(&format!("{} 0 0 0", Spacing::XL))
        .build();

    let section_heading_style = StyleBuilder::new()
        .font_size(Typography::H2)
        .font_weight("600")
        .color(ColorScheme::LIGHT.text)
        .margin("0 0 10px 0")
        .build();

    let disclaimer_text_style = StyleBuilder::new()
        .color(ColorScheme::LIGHT.text)
        .margin("0 0 10px 0")
        .line_height(Typography::LINE_HEIGHT)
        .build();

    let disclaimer_link_style = StyleBuilder::new()
        .color(ColorScheme::LIGHT.accent)
        .text_decoration("underline")
        .build();

    rsx! {
        DocumentHead {
            title: "Dioxus Experiments".to_string(),
            description: Some("Open-source Rust/WASM experiments built with Dioxus: CX-SMILES Yoga, JSON Count-rs, MGF Precursor Erro-rs, Lipid Selecto-rs, and Smellfish-rs.".to_string()),
            lang: "en".to_string(),
            webmcp: Some(WebMcpConfig {
                app_id: "index",
                title: "Dioxus Experiments",
                description: "Landing page linking the client-side Rust/WebAssembly applications in this workspace",
                inputs: &[],
                outputs: &["experiment_links"],
                edam: EdamAnnotations {
                    topics: &[
                        EdamTerm { id: "topic_3332", label: "Computational chemistry" },
                        EdamTerm { id: "topic_2258", label: "Cheminformatics" },
                        EdamTerm { id: "topic_3172", label: "Metabolomics" },
                    ],
                    operations: &[EdamTerm { id: "operation_2422", label: "Data retrieval" }],
                    input_data: &[],
                    input_formats: &[],
                    output_data: &[EdamTerm { id: "data_2048", label: "Report" }],
                    output_formats: &[EdamTerm { id: "format_2331", label: "HTML" }],
                },
            }),
            og_type: Some("website".to_string()),
            og_site_name: Some("Dioxus Experiments".to_string()),
            theme_colors: Some((ColorScheme::LIGHT.bg, ColorScheme::LIGHT.bg)),
        }

        div { style: container_style,
            skip_link {}

            Header {
                title: "Dioxus Experiments".to_string(),
                subtitle: Some("Open-source Rust/WebAssembly scientific applications built with Dioxus.".to_string()),
            }

            main { id: "main-content", style: main_style,
                section { aria_labelledby: "apps-heading",
                    h2 { id: "apps-heading", style: &section_heading_style, "Applications" }
                    div { style: grid_style,
                        {APPS.iter().map(|&app| {
                            rsx! {
                                AppCard { key: "{app.id}", app }
                            }
                        })}
                    }
                }

                aside { aria_labelledby: "disclaimer-heading", style: disclaimer_style,
                    h2 { id: "disclaimer-heading", style: &section_heading_style,
                        "About These Prototypes"
                    }
                    p { style: &disclaimer_text_style,
                        "These are "
                        strong { "experimental applications" }
                        " built with "
                        a { style: &disclaimer_link_style, href: "https://dioxuslabs.com", target: "_blank", rel: "noopener noreferrer", "Dioxus" }
                        " to explore a few things I am interested in. They're hosted here at my own discretion."
                    }
                    p { style: &disclaimer_text_style,
                        "If you're interested in hosting or collaborating on any of these projects, please "
                        a { style: &disclaimer_link_style, href: "https://github.com/adafede", target: "_blank", rel: "noopener noreferrer", "reach out on GitHub" }
                        ". I'm always open to feedback and partnership opportunities."
                    }
                }
            }

            Footer {}
        }
        SimpleAnalyticsBadge {
            domain: "adafede.github.io".to_string(),
            path: "/dioxus/".to_string(),
        }
    }
}

/// Curated list of experiment applications.
const APPS: &[AppInfo] = &[
    AppInfo {
        id: "cxsmiles-yoga",
        title: "CX-SMILES Yoga",
        path: "./cxsmiles-yoga/",
        description: "Generate a single CX-SMILES from a list of related structures by collapsing positional isomers (m: blocks) and variable-length repeats (Sg:n:) via maximum common substructure, with round-trip verification.",
    },
    AppInfo {
        id: "json-count-rs",
        title: "JSON Count-rs",
        path: "./json-count-rs/",
        description: "Stream gigabyte-scale JSON files in the browser and count booleans, numbers, strings, and nulls with zero server upload.",
    },
    AppInfo {
        id: "mgf-precursor-erro-rs",
        title: "MGF Precursor Erro-rs",
        path: "./mgf-precursor-erro-rs/",
        description: "Analyze mass spectrometry data. Visualize precursor mass errors from MGF files in absolute (Dalton) and relative (ppm) units with interactive plots.",
    },
    AppInfo {
        id: "lipid-selecto-rs",
        title: "Lipid Selecto-rs",
        path: "./lipid-selecto-rs/",
        description: "Filter and visualize lipid mass spectrometry data by chemical class. Interactive structure viewer for lipidomics analysis with support for SMILES and SMARTS queries.",
    },
    AppInfo {
        id: "smellfish-rs",
        title: "Smellfish-rs",
        path: "./smellfish-rs/",
        description: "Drop SMILES lists or CSVs, render molecules with RDKit.js, compute Ertl NP-likeness scores and motif balance, and query LOTUS and PubChem via SPARQL.",
    },
];

fn main() {
    launch(app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_non_empty_and_well_formed() {
        assert_ne!(APPS, []);
        for entry in APPS {
            assert_ne!(entry.id, "");
            assert_ne!(entry.title, "");
            assert!(entry.path.starts_with("./") && entry.path.ends_with('/'));
            assert_ne!(entry.description, "");
        }
    }

    #[test]
    fn app_renders_every_catalog_entry() {
        let mut dom = VirtualDom::new(app);
        let rendered = format!("{:?}", dom.rebuild_to_vec());
        for entry in APPS {
            assert!(
                rendered.contains(entry.path),
                "landing page missing link for {} ({rendered})",
                entry.id
            );
        }
    }

    #[test]
    fn app_card_renders_article_link_title_and_description() {
        let mut dom = VirtualDom::new(|| {
            let app = AppInfo {
                id: "demo",
                title: "Demo Tool",
                path: "./demo/",
                description: "Inspect demo data",
            };
            rsx! {
                AppCard { app }
            }
        });
        let rendered = format!("{:?}", dom.rebuild_to_vec());
        assert!(
            rendered.contains("Demo Tool")
                && rendered.contains("./demo/")
                && rendered.contains("Inspect demo data"),
            "AppCard output missing fields: {rendered}"
        );
    }
}
