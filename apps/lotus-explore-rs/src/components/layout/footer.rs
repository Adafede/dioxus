// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::hooks::use_locale;
use crate::i18n::{Locale, TextKey, t};
use dioxus::prelude::*;

// Styling lives in the component (utility classes) rather than global CSS.
// Class names (footer-line, footer-label, …) are kept as semantic hooks.
const FOOTER_LINE_ROW: &str = "footer-line flex flex-col gap-3 py-1 border-b border-border last:border-b-0 min-[640px]:flex-row min-[640px]:flex-wrap min-[640px]:items-start min-[640px]:gap-x-6 min-[640px]:gap-y-0";
const FOOTER_ROW: &str = "footer-row flex items-center gap-2 py-0.5 flex-wrap min-[640px]:flex-[1_1_280px] min-[640px]:min-w-[280px]";
const FOOTER_LABEL: &str = "footer-label inline-flex items-center font-bold uppercase tracking-[0.06em] whitespace-nowrap leading-normal px-2 py-1 rounded-lg border border-[color-mix(in_srgb,currentColor_30%,var(--border))] border-l-4 border-current bg-[color-mix(in_srgb,currentColor_14%,var(--surface))] min-h-[34px] text-sm max-[480px]:whitespace-normal max-[480px]:text-micro max-[480px]:px-1.5 max-[480px]:py-[2px] max-[480px]:min-h-0";
const FOOTER_LINKS: &str = "footer-links flex flex-wrap items-center gap-x-2.5 gap-y-1 list-none m-0 p-0 flex-none min-w-0";
const FOOTER_LI: &str = "inline-flex items-center gap-[5px] shrink-0 min-[641px]:whitespace-nowrap";
const FOOTER_LINK: &str = "footer-link no-underline text-ui leading-[1.45] min-h-[34px] inline-flex items-center px-2 py-1 rounded-[2px] hover:underline hover:bg-[color-mix(in_srgb,currentColor_8%,var(--surface))] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/40 focus-visible:rounded-[2px] max-[480px]:px-1.5 max-[480px]:py-[3px] max-[480px]:text-micro max-[480px]:min-h-[32px]";
const FOOTER_ASIDE: &str = "footer-aside text-subtle whitespace-nowrap";

#[component]
pub fn Footer() -> Element {
    let locale = use_locale();
    rsx! {
        div {
            class: "{FOOTER_LINE_ROW}",
            FooterRow {
                label: t(locale, TextKey::FooterArchive),
                label_class: "text-wd-compound",
                links: &[("https://doi.org/10.5281/zenodo.5794106", "LOTUS Frozen")],
            }
            FooterCitationRow { locale }
        }
        div {
            class: "{FOOTER_LINE_ROW}",
            FooterRow {
                label: t(locale, TextKey::FooterCode),
                label_class: "text-wd-taxon",
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
                links: &[
                    ("https://www.wikidata.org/wiki/Q104225190", "LOTUS Initiative"),
                    ("https://www.wikidata.org/", "Wikidata"),
                ],
            }
        }
        div {
            class: "{FOOTER_LINE_ROW}",
            FooterRow {
                label: t(locale, TextKey::FooterPrograms),
                label_class: "text-wd-reference",
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

#[component]
fn FooterCitationRow(locale: Locale) -> Element {
    rsx! {
        div {
            class: "{FOOTER_ROW}",
            span {
                class: "{FOOTER_LABEL} text-wd-compound",
                "{t(locale, TextKey::FooterCitation)}"
            }
            ul {
                class: "{FOOTER_LINKS}",
                role: "list",
                li {
                    class: "{FOOTER_LI}",
                    a {
                        class: "{FOOTER_LINK} font-medium text-wd-compound",
                        href: "https://doi.org/10.7554/eLife.70780",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "LOTUS Article"
                    }
                }
                li {
                    class: "{FOOTER_LI}",
                    a {
                        class: "{FOOTER_LINK} font-medium text-wd-compound",
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
            class: "{FOOTER_ROW}",
            span {
                class: "{FOOTER_LABEL} text-wd-entries",
                "{t(locale, TextKey::FooterLicense)}"
            }
            ul {
                class: "{FOOTER_LINKS}",
                role: "list",
                li {
                    class: "{FOOTER_LI}",
                    a {
                        class: "{FOOTER_LINK} font-medium text-wd-entries",
                        href: "https://creativecommons.org/publicdomain/zero/1.0/",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "CC0 1.0"
                    }
                    span {
                        class: "{FOOTER_ASIDE} text-micro",
                        "({t(locale, TextKey::FooterForData)})"
                    }
                }
                li {
                    class: "{FOOTER_LI}",
                    a {
                        class: "{FOOTER_LINK} font-medium text-wd-entries",
                        href: "https://www.gnu.org/licenses/agpl-3.0.html",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "AGPL-3.0"
                    }
                    span {
                        class: "{FOOTER_ASIDE} text-micro",
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
    links: &'static [(&'static str, &'static str)],
) -> Element {
    rsx! {
        div {
            class: "{FOOTER_ROW}",
            span {
                class: "{FOOTER_LABEL} {label_class}",
                "{label}"
            }
            ul {
                class: "{FOOTER_LINKS}",
                role: "list",
                for (href, text) in links.iter() {
                    li {
                        key: "{href}",
                        class: "{FOOTER_LI}",
                        a {
                            class: "{FOOTER_LINK} font-medium {label_class}",
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
