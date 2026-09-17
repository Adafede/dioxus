// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Welcome screen shown before the first search, with example queries.

use crate::components::copy_button::CopyButton;
use crate::components::ui::Card;
use crate::features::explore::absolute_current_url_with_query;
use crate::i18n::{TextKey, t};
use crate::ui::classes;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn WelcomeScreen() -> Element {
    let locale = crate::hooks::use_locale();
    rsx! {
        section {
            class: "page-section w-full max-w-none px-0",
            div { class: "w-full",
                div { class: "flex flex-col gap-3 px-4 py-6 sm:px-6 sm:py-8",
                    p {
                        class: "text-body leading-relaxed text-muted",
                        "{t(locale, TextKey::WelcomeLeadA)}"
                        "{t(locale, TextKey::WelcomeLeadB)}"
                        a {
                            href: "https://www.wikidata.org/wiki/Q104225190",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "mx-1 {classes::LINK}",
                            "LOTUS initiative"
                        }
                        "{t(locale, TextKey::WelcomeLeadC)}"
                        a {
                            href: "https://www.wikidata.org/",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "mx-1 {classes::LINK}",
                            "Wikidata"
                        }
                        "{t(locale, TextKey::WelcomeLeadD)}"
                        a {
                            href: "https://qlever.dev/wikidata",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            class: "mx-1 {classes::LINK}",
                            "QLever"
                        }
                        "{t(locale, TextKey::WelcomeLeadE)}"
                        " "
                        span {
                            class: "text-ui italic text-subtle",
                            "{t(locale, TextKey::LabelLanguagePolicy)}"
                        }
                    }
                }

                div { class: "h-4" }
                div { class: "px-4 sm:px-6",
                    Card {
                        class: "flex flex-col gap-3 pb-4 sm:pb-6",
                    div { class: "flex flex-col gap-1",
                        p {
                            class: "{classes::SUPPORT}",
                            "{t(locale, TextKey::WelcomeProgrammaticDownload)}"
                        }
                    }
                    div {
                        class: "mt-1 grid grid-cols-1 gap-2.5 md:grid-cols-2",
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
        div {
            role: "status",
            class: "flex items-center gap-2 {classes::RADIUS_CARD} border border-border bg-bg p-2 text-ui shadow-xs",
            span {
                class: "shrink-0 {classes::RADIUS_PILL} bg-accent/12 px-2 py-0.5 text-micro font-semibold text-accent",
                "{format}"
            }
            input {
                r#type: "text",
                readonly: true,
                value: "{absolute}",
                aria_label: "{format}",
                class: "min-w-0 flex-1 truncate {classes::RADIUS_SM_CTRL} border border-border bg-surface px-2 py-1 font-mono text-ui text-muted shadow-xs focus:outline-none {classes::FOCUS_RING}",
            }
            CopyButton { text: absolute.clone(), locale }
        }
    }
}

#[component]
fn ExRow(value: &'static str, note: &'static str) -> Element {
    rsx! {
        li {
            class: "flex flex-col gap-1 {classes::RADIUS_CARD} border border-border bg-surface p-3 shadow-xs",
            span {
                class: "self-start {classes::RADIUS_PILL} bg-accent/10 px-2 py-0.5 font-mono text-ui font-semibold text-accent",
                "{value}"
            }
            span {
                class: "text-ui text-muted",
                "{note}"
            }
        }
    }
}
