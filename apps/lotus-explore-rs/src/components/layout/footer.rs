// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::hooks::use_locale;
use crate::i18n::{Locale, TextKey, t};
use dioxus::prelude::*;

#[component]
pub fn Footer() -> Element {
    let locale = use_locale();
    rsx! {
        footer {
            class: "app-footer",
            div {
            class: "footer-line",
                FooterRow {
                    label: t(locale, TextKey::FooterArchive),
                    label_class: "text-wd-compound",
                    color_class: "text-wd-compound",
                    links: &[("https://doi.org/10.5281/zenodo.5794106", "LOTUS Frozen")],
                }
                FooterCitationRow { locale }
            }
            div {
                class: "footer-line",
                FooterRow {
                    label: t(locale, TextKey::FooterCode),
                    label_class: "text-wd-taxon bg-stat-taxon border-l-4 border-l-wd-taxon",
                    color_class: "text-wd-taxon",
                    links: &[
                        (
                            "https://github.com/Adafede/dioxus/tree/main/apps/lotus-explore-rs",
                            "lotus-explore-rs",
                        ),
                    ],
                }
                FooterRow {
                    label: t(locale, TextKey::FooterData),
                    label_class: "text-wd-taxon",
                    color_class: "text-wd-taxon",
                    links: &[
                        ("https://www.wikidata.org/wiki/Q104225190", "LOTUS Initiative"),
                        ("https://www.wikidata.org/", "Wikidata"),
                    ],
                }
            }
            div {
                class: "footer-line",
                FooterRow {
                    label: t(locale, TextKey::FooterPrograms),
                    label_class: "text-wd-reference",
                    color_class: "text-wd-reference",
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
        div {
            class: "footer-row",
            span {
                class: "footer-label text-wd-compound font-bold",
                "{t(locale, TextKey::FooterCitation)}"
            }
            ul {
                class: "footer-links",
                role: "list",
                li {
                    a {
                        class: "footer-link text-wd-compound hover:underline font-medium",
                        href: "https://doi.org/10.7554/eLife.70780",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "LOTUS Article"
                    }
                }
                li {
                    a {
                        class: "footer-link text-wd-compound hover:underline font-medium",
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
        div {
            class: "footer-row",
            span {
                class: "footer-label text-wd-entries font-bold",
                "{t(locale, TextKey::FooterLicense)}"
            }
            ul {
                class: "footer-links",
                role: "list",
                li {
                    a {
                        class: "footer-link text-wd-entries hover:underline font-medium",
                        href: "https://creativecommons.org/publicdomain/zero/1.0/",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "CC0 1.0"
                    }
                    span {
                        class: "footer-aside text-subtle text-[11px]",
                        "({t(locale, TextKey::FooterForData)})"
                    }
                }
                li {
                    a {
                        class: "footer-link text-wd-entries hover:underline font-medium",
                        href: "https://www.gnu.org/licenses/agpl-3.0.html",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "AGPL-3.0"
                    }
                    span {
                        class: "footer-aside text-subtle text-[11px]",
                        "({t(locale, TextKey::FooterForCode)})"
                    }
                }
            }
        }
    }
}

#[component]
fn FooterRow(
    label: &'static str,
    label_class: &'static str,
    color_class: &'static str,
    links: &'static [(&'static str, &'static str)],
) -> Element {
    rsx! {
        div {
            class: "footer-row",
            span {
                class: "footer-label {label_class} font-bold",
                "{label}"
            }
            ul {
                class: "footer-links",
                role: "list",
                for (href, text) in links.iter() {
                    li {
                        key: "{href}",
                        a {
                            class: "footer-link {color_class} hover:underline font-medium",
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
