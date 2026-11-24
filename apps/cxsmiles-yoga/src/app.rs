// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the cxsmiles-yoga project

//! Dioxus UI for `cxsmiles-yoga`.
//!
//! Thin reactive shell over `cxsmiles-core` and `chem-depict`.

#![expect(
    clippy::empty_enums,
    reason = "Dioxus Props derive macro generates an empty helper enum"
)]

mod clipboard;
mod controls;
mod results;

use cxsmiles_core::{CxOptions, CxResult, generate_with_options};
use dioxus::prelude::*;
use ui::{
    Button, ButtonVariant, ColorScheme, DocumentHead, EdamAnnotations, EdamTerm, Footer, Header,
    NoticeBar, NoticeTone, SegmentedControl, SimpleAnalyticsBadge, StyleBuilder, Typography,
    WebMcpConfig, skip_link,
};

use self::controls::{InputCard, example_items};
use self::results::ResultsPanel;
use crate::examples::example_smiles;

const WEBMCP_CONFIG: WebMcpConfig = WebMcpConfig {
    app_id: "cxsmiles-yoga",
    title: "CX-SMILES Yoga",
    description: "Web app for generating CX-SMILES from lists of related structures — collapsing positional isomers (m: blocks) and variable-length repeats (Sg:n: blocks) via maximum common substructure",
    inputs: &["smiles_list"],
    outputs: &["cx_smiles"],
    edam: EdamAnnotations {
        topics: &[
            EdamTerm {
                id: "topic_2258",
                label: "Cheminformatics",
            },
            EdamTerm {
                id: "topic_0154",
                label: "Small molecules",
            },
        ],
        operations: &[
            EdamTerm {
                id: "operation_0366",
                label: "Structure clustering",
            },
            EdamTerm {
                id: "operation_0335",
                label: "Formatting",
            },
            EdamTerm {
                id: "operation_0570",
                label: "Structure visualisation",
            },
        ],
        input_data: &[EdamTerm {
            id: "data_2301",
            label: "SMILES string",
        }],
        input_formats: &[EdamTerm {
            id: "format_1196",
            label: "SMILES",
        }],
        output_data: &[
            EdamTerm {
                id: "data_2301",
                label: "SMILES string",
            },
            EdamTerm {
                id: "data_1710",
                label: "Structure image",
            },
        ],
        output_formats: &[
            EdamTerm {
                id: "format_1196",
                label: "SMILES",
            },
            EdamTerm {
                id: "format_3604",
                label: "SVG",
            },
        ],
    },
};

/// Main `cxsmiles-yoga` UI component.
///
/// # Errors
///
/// Returns a Dioxus render element propagating any `rsx!` render error.
pub fn app() -> Element {
    let mut input = use_signal(String::new);
    let mut result = use_signal(|| None::<CxResult>);
    let mut error = use_signal(|| None::<String>);
    let mut selected = use_signal(String::new);
    let aromatic = use_signal(|| false);
    let light = ColorScheme::LIGHT;

    let on_generate = move |_| {
        error.set(None);
        result.set(None);
        let lines: Vec<String> = input
            .read()
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_owned)
            .collect();
        let options = CxOptions {
            expand_aromatic: *aromatic.read(),
        };
        match generate_with_options(&lines, options) {
            Ok(r) => result.set(Some(r)),
            Err(e) => error.set(Some(e.to_string())),
        }
    };

    let on_example_select = move |value: String| {
        if let Some(sm) = example_smiles(&value) {
            input.set(sm.to_string());
        }
        selected.set(value);
        result.set(None);
        error.set(None);
    };

    rsx! {
        DocumentHead {
            title: "🧘 CX-SMILES Yoga".to_string(),
            webmcp: Some(WEBMCP_CONFIG),
            lang: "en".to_string(),
            theme_colors: Some((light.bg, ColorScheme::DARK.bg)),
        }

        skip_link {}

        Header {
            title: "🧘 CX-SMILES Yoga".to_string(),
            subtitle: Some("Generate CX-SMILES from a list of related structures".to_string()),
        }

        main { id: "main-content", style: StyleBuilder::new().property("max-width", "1000px").margin("0 auto").padding("1.5rem 1rem 3rem").build(),
            p { style: StyleBuilder::new().color(light.text2).font_size(Typography::BODY).property("max-width", "60rem").build(),
                "Paste a list of related SMILES (one per line). The tool finds the shared scaffold, "
                "classifies each variable region as a positional isomer (m:) or a variable-length "
                "repeat (Sg:n:), and emits a single CX-SMILES. It then re-expands the result and "
                "reports how many of your inputs round-trip — the higher the percentage, the more "
                "confident the output."
            }

            div { style: StyleBuilder::new().margin("1rem 0").build(),
                SegmentedControl {
                    aria_label: "Pick an example".to_string(),
                    selected_value: selected.read().clone(),
                    items: example_items(),
                    on_select: on_example_select,
                }
            }

            InputCard { input, aromatic }

            div { style: StyleBuilder::new().property("margin-bottom", "1rem").build(),
                Button {
                    label: "Generate",
                    variant: ButtonVariant::Primary,
                    disabled: input.read().trim().is_empty(),
                    onclick: Some(EventHandler::new(on_generate)),
                }
            }

            if let Some(err) = error.read().as_ref() {
                NoticeBar { label: err.clone(), tone: NoticeTone::Danger }
            }

            ResultsPanel { result }
        }
        Footer {}
        SimpleAnalyticsBadge {
            domain: "adafede.github.io".to_string(),
            path: "/dioxus/cxsmiles-yoga/".to_string(),
        }
    }
}
