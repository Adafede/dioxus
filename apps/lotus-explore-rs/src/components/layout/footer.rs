// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::hooks::use_locale;
use crate::i18n::{Locale, TextKey, t};
use dioxus::prelude::*;
use ui::prelude::*;
use ui::styles::lotus::tokens::{
    FOOTER_WD_COMPOUND, FOOTER_WD_REFERENCE, FOOTER_WD_TAXON,
};

#[component]
pub fn Footer() -> Element {
    let locale = use_locale();
    rsx! {
        footer { class: "app-footer", style: footer_style(),
            div { class: "footer-line", style: footer_line_style(),
                FooterRow {
                    label: t(locale, TextKey::FooterArchive),
                    color: FOOTER_WD_COMPOUND,
                    links: &[("https://doi.org/10.5281/zenodo.5794106", "LOTUS Frozen"),],
                }
                FooterCitationRow { locale }
            }
            div { class: "footer-line", style: footer_line_style(),
                FooterRow {
                    label: t(locale, TextKey::FooterCode),
                    color: FOOTER_WD_TAXON,
                    links: &[
                        (
                            "https://github.com/Adafede/dioxus/tree/main/apps/lotus-explore-rs",
                            "lotus-explore-rs",
                        ),
                    ],
                }
                FooterRow {
                    label: t(locale, TextKey::FooterData),
                    color: FOOTER_WD_TAXON,
                    links: &[
                        ("https://www.wikidata.org/wiki/Q104225190", "LOTUS Initiative"),
                        ("https://www.wikidata.org/", "Wikidata"),
                    ],
                }
            }
            div { class: "footer-line", style: footer_line_style(),
                FooterRow {
                    label: t(locale, TextKey::FooterPrograms),
                    color: FOOTER_WD_REFERENCE,
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

fn footer_style() -> String {
    StyleBuilder::new()
        .display("flex")
        .flex_direction("column")
        .background_color("var(--panel-bg)")
        .property("border-top", "1px solid var(--panel-border)")
        .property("margin-top", "auto")
        .padding("16px 28px 20px")
        .build()
}

#[component]
fn FooterCitationRow(locale: Locale) -> Element {
    rsx! {
        div { class: "footer-row", style: footer_row_style(),
            span { class: "footer-label", style: footer_label_style(), "{t(locale, TextKey::FooterCitation)}" }
            ul { class: "footer-links", role: "list", style: footer_links_style(),
                li {
                    a {
                        class: "footer-link",
                        href: "https://doi.org/10.7554/eLife.70780",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        style: footer_link_style(FOOTER_WD_COMPOUND),
                        "LOTUS Article"
                    }
                }
                li {
                    a {
                        class: "footer-link",
                        href: "/docs/references.bib",
                        download: "references.bib",
                        style: footer_link_style(FOOTER_WD_COMPOUND),
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
        div { class: "footer-row", style: footer_row_style(),
            span { class: "footer-label", style: footer_label_style(), "{t(locale, TextKey::FooterLicense)}" }
            ul { class: "footer-links", role: "list", style: footer_links_style(),
                li {
                    a {
                        class: "footer-link",
                        href: "https://creativecommons.org/publicdomain/zero/1.0/",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        style: footer_link_style(FOOTER_WD_REFERENCE),
                        "CC0 1.0"
                    }
                    span { class: "footer-aside", style: footer_aside_style(), "{t(locale, TextKey::FooterForData)}" }
                }
                li {
                    a {
                        class: "footer-link",
                        href: "https://www.gnu.org/licenses/agpl-3.0.html",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        style: footer_link_style(FOOTER_WD_REFERENCE),
                        "AGPL-3.0"
                    }
                    span { class: "footer-aside", style: footer_aside_style(), "{t(locale, TextKey::FooterForCode)}" }
                }
            }
        }
    }
}

#[component]
fn FooterRow(
    label: &'static str,
    color: &'static str,
    links: &'static [(&'static str, &'static str)],
) -> Element {
    rsx! {
        div { class: "footer-row", style: footer_row_style(),
            span { class: "footer-label", style: footer_label_style(), "{label}" }
            ul { class: "footer-links", role: "list", style: footer_links_style(),
                for (href, text) in links.iter() {
                    li {
                        a {
                            class: "footer-link",
                            href: "{href}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            style: footer_link_style(color),
                            "{text}"
                        }
                    }
                }
            }
        }
    }
}

fn footer_line_style() -> String {
    StyleBuilder::new()
        .display("grid")
        .property(
            "grid-template-columns",
            "repeat(auto-fit, minmax(300px, 1fr))",
        )
        .gap("0 24px")
        .align_items("start")
        .padding("10px 0")
        .border_bottom("1px solid var(--panel-border)")
        .build()
}

fn footer_row_style() -> String {
    StyleBuilder::new()
        .display("grid")
        .property(
            "grid-template-columns",
            "clamp(7.5rem, 7vw, 9rem) minmax(0, 1fr)",
        )
        .gap("4px 6px")
        .align_items("start")
        .padding("2px 0")
        .build()
}

fn footer_label_style() -> String {
    StyleBuilder::new()
        .color("var(--text2)")
        .font_weight("700")
        .property("text-transform", "uppercase")
        .font_size("var(--fs-0)")
        .property("letter-spacing", "1px")
        .property("min-width", "0")
        .property("white-space", "nowrap")
        .property("padding-top", "4px")
        .text_align("left")
        .build()
}

fn footer_aside_style() -> String {
    StyleBuilder::new()
        .color("var(--text2)")
        .font_size("var(--fs-0)")
        .build()
}

fn footer_links_style() -> String {
    StyleBuilder::new()
        .property("list-style", "none")
        .display("flex")
        .property("flex-wrap", "wrap")
        .gap("3px 5px")
        .property("margin", "0")
        .property("padding", "0")
        .property("min-width", "0")
        .justify_content("flex-start")
        .align_items("flex-start")
        .build()
}

fn footer_link_style(color: &str) -> String {
    StyleBuilder::new()
        .color(color)
        .text_decoration("none")
        .font_weight("700")
        .border("1px solid var(--border)")
        .padding("5px 10px")
        .border_radius("999px")
        .property("display", "inline-block")
        .property("transition", "border-color .15s ease, background .15s ease")
        .build()
}
