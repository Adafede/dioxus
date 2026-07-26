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
/// # use index::app;
/// #[component]
/// pub fn MyApp() -> Element {
///     rsx! { app {} }
/// }
/// ```
#[component]
pub fn app() -> Element {
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
        div { style: container_style,
            Header {
                title: "🦀 Dioxus Experiments".to_string(),
                subtitle: Some("A collection of open-source Rust/WASM applications testing the boundaries of what's possible on the web. From knowledge graphs to mass spectrometry analysis.".to_string()),
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
                        " to explore a few things I am interested in. They're hosted here at my own discretion."
                    }
                    p {
                        "If you're interested in hosting or collaborating on any of these projects, please "
                        a { href: "https://github.com/adafede", target: "_blank", rel: "noopener noreferrer", "reach out on GitHub" }
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
        description: "Query the LOTUS knowledge graph powered by Wikidata. Explore natural products, their taxons, and bibliographic references through an interactive search interface.",
    },
    AppInfo {
        id: "jsoncount",
        title: "🧮 JSON Counter",
        path: "./jsoncount/",
        description: "Streaming JSON analysis for large datasets. Upload gigabyte-scale files and get instant statistics on field cardinality and null distributions.",
    },
    AppInfo {
        id: "mgf-precursor-error",
        title: "📊 MGF Precursor Error",
        path: "./mgf-precursor-erro-rs/",
        description: "Analyze mass spectrometry data. Visualize precursor mass errors from MGF files in absolute (Dalton) and relative (ppm) units with interactive plots.",
    },
];

fn main() {
    dioxus::launch(app);
}
