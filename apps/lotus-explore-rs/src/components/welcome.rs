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
            class: "mx-auto flex w-full max-w-4xl flex-col gap-6 p-6",
            div {
                class: "flex flex-col gap-3 rounded-2xl border border-panel-border bg-panel-soft p-6 shadow-xs",
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
                        class: "mt-2 block text-ui italic text-subtle",
                        "{t(locale, TextKey::LabelLanguagePolicy)}"
                    }
                }
            }

            div {
                class: "flex flex-col gap-3",
                ul {
                    class: "grid grid-cols-1 gap-3 md:grid-cols-2",
                    ExRow {
                        value: "taxon=<name|QID|*>",
                        note: t(locale, TextKey::ExampleGentiana),
                    }
                    ExRow {
                        value: "structure=<SMILES|Molfile>",
                        note: t(locale, TextKey::ExampleSmilesOnly),
                    }
                }
            }

            Card {
                class: "flex flex-col gap-3",
                div { class: "flex flex-col gap-1",
                    p {
                        class: "{classes::SUPPORT}",
                        "{t(locale, TextKey::WelcomeProgrammaticDownload)}"
                    }
                }
                div {
                    class: "mt-1 flex flex-col gap-2.5",
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
        div {
            role: "status",
            class: "flex items-center gap-2 rounded-lg border border-border bg-bg p-2 text-ui shadow-xs",
            span {
                class: "shrink-0 rounded bg-accent/12 px-2 py-0.5 text-[11px] font-semibold text-accent",
                "{format}"
            }
            input {
                r#type: "text",
                readonly: true,
                value: "{absolute}",
                aria_label: "{format}",
                class: "min-w-0 flex-1 truncate rounded-lotus-sm border border-border bg-surface px-2 py-1 font-mono text-ui text-muted shadow-xs focus:outline-none",
            }
            CopyButton { text: absolute.clone(), locale }
        }
    }
}

#[component]
fn ExRow(value: &'static str, note: &'static str) -> Element {
    rsx! {
        li {
            class: "flex flex-col gap-1 rounded-xl border border-border bg-surface p-3 shadow-xs",
            span {
                class: "self-start rounded-md bg-accent/10 px-2 py-0.5 font-mono text-ui font-semibold text-accent",
                "{value}"
            }
            span {
                class: "text-ui text-muted",
                "{note}"
            }
        }
    }
}
