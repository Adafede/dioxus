// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::hooks::use_locale;
use crate::i18n::{Locale, TextKey, t};
use dioxus::prelude::*;

#[component]
pub fn Footer() -> Element {
    let locale = use_locale();
    rsx! {
        footer { class: "app-footer",
            div { class: "footer-line",
                FooterRow {
                    label: t(locale, TextKey::FooterArchive),
                    class: "footer-link red",
                    links: &[("https://doi.org/10.5281/zenodo.5794106", "Frozen version (Zenodo)"),],
                }
                FooterCitationRow { locale }
            }
            div { class: "footer-line",
                FooterRow {
                    label: t(locale, TextKey::FooterCode),
                    class: "footer-link green",
                    links: &[
                        (
                            "https://github.com/Adafede/dioxus/tree/main/apps/lotus-explorer",
                            "lotus-explorer",
                        ),
                    ],
                }
                FooterRow {
                    label: t(locale, TextKey::FooterData),
                    class: "footer-link green",
                    links: &[
                        ("https://www.wikidata.org/wiki/Q104225190", "LOTUS Initiative"),
                        ("https://www.wikidata.org/", "Wikidata"),
                    ],
                }
            }
            div { class: "footer-line",
                FooterRow {
                    label: t(locale, TextKey::FooterPrograms),
                    class: "footer-link blue",
                    links: &[
                        ("https://github.com/cdk/depict", "CDK Depict"),
                        ("https://citation.js.org", "Citation.js"),
                        ("https://lifescience.opensource.epam.com/ketcher", "Ketcher"),
                        ("https://qlever.dev/wikidata", "QLever"),
                        ("https://www.rdkitjs.com", "RDKit.js"),
                        ("https://doi.org/10.1186/s13321-018-0282-y", "Sachem"),
                    ],
                }
                FooterLicenseRow { locale }
            }
        }
    }
}

#[component]
fn FooterCitationRow(locale: Locale) -> Element {
    rsx! {
        div { class: "footer-row",
            span { class: "footer-label", "{t(locale, TextKey::FooterCitation)}" }
            ul { class: "footer-links", role: "list",
                li {
                    a {
                        class: "footer-link red",
                        href: "https://doi.org/10.7554/eLife.70780",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "LOTUS paper (eLife)"
                    }
                }
                li {
                    a {
                        class: "footer-link red",
                        href: "/docs/references.bib",
                        download: "references.bib",
                        "BibTeX"
                    }
                }
            }
        }
    }
}

#[component]
fn FooterLicenseRow(locale: Locale) -> Element {
    rsx! {
        div { class: "footer-row",
            span { class: "footer-label", "{t(locale, TextKey::FooterLicense)}" }
            ul { class: "footer-links", role: "list",
                li {
                    a {
                        class: "footer-link blue",
                        href: "https://creativecommons.org/publicdomain/zero/1.0/",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "CC0 1.0"
                    }
                    span { class: "footer-aside", "{t(locale, TextKey::FooterForData)}" }
                }
                li {
                    a {
                        class: "footer-link blue",
                        href: "https://www.gnu.org/licenses/agpl-3.0.html",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "AGPL-3.0"
                    }
                    span { class: "footer-aside", "{t(locale, TextKey::FooterForCode)}" }
                }
            }
        }
    }
}

#[component]
fn FooterRow(
    label: &'static str,
    class: &'static str,
    links: &'static [(&'static str, &'static str)],
) -> Element {
    rsx! {
        div { class: "footer-row",
            span { class: "footer-label", "{label}" }
            ul { class: "footer-links", role: "list",
                for (href, text) in links.iter() {
                    li {
                        a {
                            class: "{class}",
                            href: "{href}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "{text}"
                        }
                    }
                }
            }
        }
    }
}
