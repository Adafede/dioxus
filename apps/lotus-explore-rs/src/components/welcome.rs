// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Welcome screen shown before the first search, with example queries.

use crate::components::copy_button::CopyButton;
use crate::features::explore::absolute_current_url_with_query;
use crate::i18n::{TextKey, t};
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn WelcomeScreen() -> Element {
    let locale = crate::hooks::use_locale();
    rsx! {
        section { class: "welcome",
            div { class: "welcome-hero",
                p { class: "welcome-lead",
                    "{t(locale, TextKey::WelcomeLeadA)}"
                    "{t(locale, TextKey::WelcomeLeadB)}"
                    a {
                        class: "welcome-inline-link",
                        href: "https://www.wikidata.org/wiki/Q104225190",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "LOTUS initiative"
                    }
                    "{t(locale, TextKey::WelcomeLeadC)}"
                    a {
                        class: "welcome-inline-link",
                        href: "https://www.wikidata.org/",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "Wikidata"
                    }
                    "{t(locale, TextKey::WelcomeLeadD)}"
                    a {
                        class: "welcome-inline-link",
                        href: "https://qlever.dev/wikidata",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "QLever"
                    }
                    "{t(locale, TextKey::WelcomeLeadE)}"
                }
                p { class: "form-hint welcome-support-text welcome-language-note",
                    "{t(locale, TextKey::LabelLanguagePolicy)}"
                }
            }

            div { class: "welcome-examples",
                ul { class: "example-list",
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
                p { class: "form-hint welcome-support-text welcome-cli-hint",
                    "{t(locale, TextKey::WelcomeProgrammaticDownload)}"
                }
                div { class: "welcome-cli-list",
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
        div { class: "notice notice-info", role: "status",
            span { class: "notice-label", "{format}" }
            input {
                class: "notice-value notice-copy-field mono",
                r#type: "text",
                readonly: true,
                value: "{absolute}",
                aria_label: "{format}",
            }
            CopyButton { text: absolute.clone(), locale }
        }
    }
}

#[component]
fn ExRow(value: &'static str, note: &'static str) -> Element {
    rsx! {
        li { class: "notice notice-info", role: "status",
            span { class: "notice-label mono", "{value}" }
            span { class: "notice-value", "{note}" }
        }
    }
}
