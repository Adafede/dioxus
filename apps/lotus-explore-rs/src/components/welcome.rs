// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Welcome screen shown before the first search, with example queries.

use crate::components::copy_button::CopyButton;
use crate::features::explore::absolute_current_url_with_query;
use crate::i18n::{TextKey, t};
use dioxus::prelude::*;
use std::sync::Arc;
use ui::prelude::*;

#[component]
pub fn WelcomeScreen() -> Element {
    let locale = crate::hooks::use_locale();
    rsx! {
        section { style: welcome_style(),
            div { style: welcome_hero_style(),
                p { style: welcome_lead_style(),
                    "{t(locale, TextKey::WelcomeLeadA)}"
                    "{t(locale, TextKey::WelcomeLeadB)}"
                    a {
                        href: "https://www.wikidata.org/wiki/Q104225190",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        style: inline_link_style(),
                        "LOTUS initiative"
                    }
                    "{t(locale, TextKey::WelcomeLeadC)}"
                    a {
                        href: "https://www.wikidata.org/",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        style: inline_link_style(),
                        "Wikidata"
                    }
                    "{t(locale, TextKey::WelcomeLeadD)}"
                    a {
                        href: "https://qlever.dev/wikidata",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        style: inline_link_style(),
                        "QLever"
                    }
                    "{t(locale, TextKey::WelcomeLeadE)}"
                    " "
                    span { style: support_text_style(),
                        "{t(locale, TextKey::LabelLanguagePolicy)}"
                    }
                }
            }

            div { style: welcome_examples_style(),
                ul { style: example_list_style(),
                    ExRow {
                        value: "taxon=<name|QID>",
                        note: t(locale, TextKey::ExampleGentiana),
                    }
                    ExRow {
                        value: "*",
                        note: t(locale, TextKey::ExampleAllTriples),
                    }
                    ExRow {
                        value: "structure=<SMILES|Molfile>",
                        note: t(locale, TextKey::ExampleSmilesOnly),
                    }
                }
                p { style: support_text_style(),
                    "{t(locale, TextKey::WelcomeProgrammaticDownload)}"
                }
                div { style: cli_list_style(),
                    DownloadExampleRow {
                        locale,
                        format: t(locale, TextKey::ExampleQueryExecute),
                        query: "?taxon=Gentiana%20lutea&execute=true",
                    }
                    DownloadExampleRow {
                        locale,
                        format: t(locale, TextKey::ExampleQueryTaxon),
                        query: "?taxon=*&download=true&format=csv",
                    }
                    DownloadExampleRow {
                        locale,
                        format: t(locale, TextKey::ExampleQueryStructure),
                        query: "?structure=c1ccccc1&structure_search_type=similarity&smiles_threshold=0.85&download=true&format=json",
                    }
                    DownloadExampleRow {
                        locale,
                        format: t(locale, TextKey::ExampleQueryAdvanced),
                        query: "?taxon=Fungi&mass_filter=true&mass_min=0&mass_max=300&year_filter=true&year_start=2000&year_end=2026&formula_filter=true&c_min=1&c_max=10&cl_state=required&br_state=excluded&download=true&format=rdf",
                    }
                }
            }
        }
    }
}

#[component]
fn DownloadExampleRow(
    locale: crate::i18n::Locale,
    format: &'static str,
    query: &'static str,
) -> Element {
    let absolute = absolute_current_url_with_query(query.trim_start_matches('?'));
    let absolute = Arc::<str>::from(absolute);
    rsx! {
        div { role: "status", style: notice_base_style(),
            span { style: notice_label_style(), "{format}" }
            input {
                r#type: "text",
                readonly: true,
                value: "{absolute}",
                aria_label: "{format}",
                style: notice_input_style(),
            }
            CopyButton { text: absolute.clone(), locale }
        }
    }
}

#[component]
fn ExRow(value: &'static str, note: &'static str) -> Element {
    rsx! {
        li { role: "status", style: notice_base_style(),
            span { style: mono_label_style(), "{value}" }
            span { style: notice_value_style(), "{note}" }
        }
    }
}

fn welcome_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("12px")
        .padding("16px 22px")
        .property("width", "100%")
        .property("max-width", "none")
        .build()
}

fn welcome_hero_style() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("min-width", "0")
        .build()
}

fn welcome_lead_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-1)")
        .color("var(--text2)")
        .property("margin-top", "6px")
        .property("line-height", "1.60")
        .property("max-width", "none")
        .property("overflow-wrap", "anywhere")
        .build()
}

fn inline_link_style() -> String {
    StyleBuilder::new()
        .text_decoration("underline")
        .property("text-underline-offset", "2px")
        .font_weight("600")
        .build()
}

fn support_text_style() -> String {
    StyleBuilder::new()
        .font_size("var(--fs-1)")
        .property("line-height", "1.55")
        .color("var(--text2)")
        .property("margin-top", "10px")
        .property("max-width", "72ch")
        .build()
}

fn welcome_examples_style() -> String {
    StyleBuilder::new()
        .property("width", "100%")
        .property("min-width", "0")
        .build()
}

fn example_list_style() -> String {
    StyleBuilder::new()
        .property("list-style", "none")
        .display("flex")
        .flex_direction("column")
        .gap("6px")
        .build()
}

fn cli_list_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .gap("8px")
        .property("margin-top", "3px")
        .build()
}

fn mono_label_style() -> String {
    StyleBuilder::new().font_family("var(--mono)").build()
}

fn notice_base_style() -> String {
    StyleBuilder::new()
        .margin("10px 24px 0")
        .padding("9px 12px")
        .display("flex")
        .flex_direction("row")
        .property("flex-wrap", "wrap")
        .align_items("center")
        .gap("12px")
        .border_radius("12px")
        .font_size("var(--fs-0)")
        .border("1px solid var(--border)")
        .background_color("var(--surface)")
        .box_shadow("var(--shadow-xs)")
        .property("min-width", "0")
        .property(
            "transition",
            "background .15s ease, border-color .15s ease, box-shadow .15s ease",
        )
        .build()
}

fn notice_label_style() -> String {
    StyleBuilder::new()
        .display("inline-flex")
        .align_items("center")
        .property("text-transform", "uppercase")
        .property("letter-spacing", "1px")
        .font_size("var(--fs-label)")
        .font_weight("700")
        .property("line-height", "1.4")
        .padding("2px 6px")
        .border_radius("3px")
        .property("flex-shrink", "0")
        .build()
}

fn notice_input_style() -> String {
    StyleBuilder::new()
        .property("flex", "1 1 auto")
        .property("min-width", "min(200px, 100%)")
        .property("max-width", "100%")
        .background_color("var(--surface)")
        .border("1px solid var(--border)")
        .border_radius("var(--radius-sm)")
        .color("var(--text2)")
        .padding("4px 8px")
        .font_size("var(--fs-0)")
        .property("overflow", "hidden")
        .property("text-overflow", "ellipsis")
        .build()
}

fn notice_value_style() -> String {
    StyleBuilder::new()
        .property("flex", "1")
        .color("var(--text)")
        .property("word-break", "break-word")
        .property("line-height", "1.4")
        .build()
}
