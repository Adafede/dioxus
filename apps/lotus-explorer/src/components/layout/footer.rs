// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::i18n::{Locale, TextKey, t};
use dioxus::prelude::*;

#[component]
pub fn Footer(locale: Locale) -> Element {
    rsx! {
        footer { class: "app-footer",
            FooterRow {
                label: t(locale, TextKey::FooterArchive),
                class: "footer-link red",
                links: &[("https://doi.org/10.5281/zenodo.5794106", "Frozen version (Zenodo)")],
            }
            div { class: "footer-row",
                span { class: "footer-label", "{t(locale, TextKey::FooterCitation)}" }
                a {
                    class: "footer-link red",
                    href: "https://doi.org/10.7554/eLife.70780",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "LOTUS paper (eLife)"
                }
                span { class: "footer-sep", "·" }
                a {
                    class: "footer-link red",
                    href: "/docs/references.bib",
                    download: "references.bib",
                    "BibTeX"
                }
            }
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
            FooterRow {
                label: t(locale, TextKey::FooterPrograms),
                class: "footer-link blue",
                links: &[
                    ("https://github.com/cdk/depict", "CDK Depict"),
                    ("https://citation.js.org", "citation.js"),
                    ("https://idsm.elixir-czech.cz", "IDSM"),
                    ("https://lifescience.opensource.epam.com/ketcher", "Ketcher"),
                    ("https://qlever.dev/wikidata", "QLever"),
                    ("https://www.rdkitjs.com", "RDKit.js"),
                    ("https://doi.org/10.1186/s13321-018-0282-y", "Sachem"),
                ],
            }
            div { class: "footer-row",
                span { class: "footer-label", "{t(locale, TextKey::FooterLicense)}" }
                a {
                    class: "footer-link blue",
                    href: "https://creativecommons.org/publicdomain/zero/1.0/",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    "CC0 1.0"
                }
                span { class: "footer-aside", "{t(locale, TextKey::FooterForData)}" }
                span { class: "footer-sep", "·" }
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

#[component]
fn FooterRow(
    label: &'static str,
    class: &'static str,
    links: &'static [(&'static str, &'static str)],
) -> Element {
    rsx! {
        div { class: "footer-row",
            span { class: "footer-label", "{label}" }
            for (i, (href, text)) in links.iter().enumerate() {
                if i > 0 {
                    span { class: "footer-sep", "·" }
                }
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
