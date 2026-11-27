// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Static footer with citation, data sources, code, literature, and license links.

use crate::literature::LITERATURE;
use dioxus::prelude::*;

/// Static footer banner with citation, data, code, references, and program links.
#[component]
pub(super) fn Footer() -> Element {
    rsx! {
        footer { class: "app-footer",
            div { class: "footer-line",
                div { class: "footer-row",
                    span { class: "footer-label", "Citation" }
                    ul { class: "footer-links", role: "list",
                        li {
                            a { class: "footer-link red", href: "https://doi.org/10.7554/eLife.70780", target: "_blank", rel: "noopener noreferrer", "LOTUS Article" }
                        }
                    }
                }
            }
            div { class: "footer-line",
                div { class: "footer-row",
                    span { class: "footer-label", "Data" }
                    ul { class: "footer-links", role: "list",
                        li { a { class: "footer-link green", href: "https://www.wikidata.org/wiki/Q104225190", target: "_blank", rel: "noopener noreferrer", "LOTUS Initiative" } }
                        li { a { class: "footer-link green", href: "https://www.wikidata.org/", target: "_blank", rel: "noopener noreferrer", "Wikidata" } }
                        li { a { class: "footer-link green", href: "https://pubchem.ncbi.nlm.nih.gov/", target: "_blank", rel: "noopener noreferrer", "PubChem" } }
                    }
                }
                div { class: "footer-row",
                    span { class: "footer-label", "Code" }
                    ul { class: "footer-links", role: "list",
                        li { a { class: "footer-link blue", href: "https://github.com/Adafede/dioxus/tree/main/apps/smellfish-rs", target: "_blank", rel: "noopener noreferrer", "smellfish-rs" } }
                    }
                }
            }
            div { class: "footer-line",
                div { class: "footer-row",
                    span { class: "footer-label", "References" }
                    ul { class: "footer-links", role: "list",
                        for paper in LITERATURE {
                            li { key: "{paper.doi}",
                                a { class: "footer-link purple", href: "https://doi.org/{paper.doi}", target: "_blank", rel: "noopener noreferrer", title: "{paper.note}", "{paper.title}" }
                            }
                        }
                    }
                }
            }
            div { class: "footer-line",
                div { class: "footer-row",
                    span { class: "footer-label", "Programs" }
                    ul { class: "footer-links", role: "list",
                        li { a { class: "footer-link blue", href: "https://qlever.dev/wikidata", target: "_blank", rel: "noopener noreferrer", "QLever" } }
                        li { a { class: "footer-link blue", href: "https://www.rdkitjs.com", target: "_blank", rel: "noopener noreferrer", "RDKit.js" } }
                    }
                }
                div { class: "footer-row",
                    span { class: "footer-label", "License" }
                    ul { class: "footer-links", role: "list",
                        li { a { class: "footer-link blue", href: "https://www.gnu.org/licenses/agpl-3.0.html", target: "_blank", rel: "noopener noreferrer", "AGPL-3.0" } }
                    }
                }
            }
        }
    }
}
