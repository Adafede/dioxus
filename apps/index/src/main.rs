//! Type-safe, accessible landing page for Dioxus experiments.
//!
//! This module provides the root application component and demonstrates best practices for:
//! - Semantic HTML with ARIA annotations
//! - Reusable, accessible component patterns
//! - Clean separation of concerns
//! - Comprehensive documentation

use dioxus::prelude::*;
use ui::prelude::*;

mod accessibility;
mod components;

use components::{AppCard, AppInfo};

/// Root application component.
///
/// Renders a fully accessible, theme-aware landing page showcasing Dioxus experiments.
/// Implements WCAG AAA accessibility standards with:
/// - Semantic HTML structure
/// - Keyboard navigation support
/// - Proper focus management
/// - Color contrast compliance
/// - Responsive design
///
/// # Example
///
/// ```rust,no_run
/// # use dioxus::prelude::*;
/// # use index::App;
/// #[component]
/// pub fn MyApp() -> Element {
///     rsx! { App {} }
/// }
/// ```
#[component]
pub fn App() -> Element {
    let container_style = StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .property("min-height", "100vh")
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

    let disclaimer_heading_style = StyleBuilder::new()
        .font_size(Typography::H2)
        .font_weight("600")
        .color(ColorScheme::LIGHT.text)
        .margin("0 0 10px 0")
        .build();

    rsx! {
        accessibility::SkipLink {}

        div { style: container_style,
            Header {
                title: "🦀 Dioxus Experiments".to_string(),
                subtitle: Some("A collection of open-source Rust/WASM applications exploring UI patterns, performance optimization, and data processing.".to_string()),
            }

            main { id: "main-content", role: "main", style: main_style,
                section { aria_labelledby: "apps-heading",
                    div { style: grid_style,
                        {APPS.iter().map(|&app| {
                            rsx! {
                                AppCard { key: "{app.id}", app }
                            }
                        })}
                    }
                }

                section { role: "complementary", aria_labelledby: "disclaimer-heading", style: disclaimer_style,
                    h2 { id: "disclaimer-heading", style: disclaimer_heading_style, "⚠️ About These Prototypes" }
                    p {
                        "These are "
                        strong { "experimental applications" }
                        " built with "
                        a { href: "https://dioxuslabs.com", target: "_blank", rel: "noopener noreferrer", "Dioxus" }
                        " to explore UI patterns, performance optimizations, and data processing workflows. They're hosted here at my own discretion."
                    }
                    p {
                        "If you're interested in hosting or collaborating on any of these projects, please "
                        a { href: "https://github.com/adrutz", target: "_blank", rel: "noopener noreferrer", "reach out on GitHub" }
                        ". I'm always open to feedback and partnership opportunities."
                    }
                }
            }

            Footer {}
        }
    }
}

/// Curated list of experiment applications.
const APPS: &[AppInfo] = &[
    AppInfo {
        id: "lotus-explorer",
        title: "🪷 LOTUS Wikidata Explorer",
        path: "./lotus-explorer/",
        description: "Browse natural product compounds with taxon references and Wikidata integration. Explore chemical data structures in an interactive knowledge graph.",
    },
    AppInfo {
        id: "jsoncount",
        title: "🧮 JSON Counter",
        path: "./jsoncount/",
        description: "Analyze JSON files efficiently. Upload any JSON document and get field-by-field statistics on non-null values, with streaming support for large files.",
    },
    AppInfo {
        id: "mgf-precursor-error",
        title: "📊 MGF Precursor Error",
        path: "./mgf-precursor-erro-rs/",
        description: "Investigate mass spectrometry data. Upload MGF files and visualize precursor mass errors in both absolute (Da) and relative (ppm) units.",
    },
    AppInfo {
        id: "hello-world",
        title: "👋 Hello World",
        path: "./hello-world/",
        description: "Getting started template for building new Dioxus applications. A minimal, clean foundation for your next WASM project.",
    },
];

fn main() {
    dioxus::launch(App);
}
