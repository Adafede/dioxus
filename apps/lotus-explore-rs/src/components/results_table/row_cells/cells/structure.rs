// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Structure (depiction image) cell for results-table rows.

use crate::components::results_table::row_cells::row_text::RowText;
use crate::i18n::{Locale, aria_chemical_structure};
use dioxus::prelude::*;

pub(in crate::components::results_table::row_cells) fn structure_cell(
    locale: Locale,
    text: RowText,
    depict_url: Option<&str>,
    name: &str,
) -> Element {
    rsx! {
        td { class: "td-depict",
            if let Some(url) = depict_url {
                a {
                    href: "{url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{text.open_full_size_depiction}",
                    img {
                        class: "depict-img",
                        src: "{url}",
                        alt: "{aria_chemical_structure(locale, name)}",
                        loading: "lazy",
                        width: "120",
                        height: "72",
                    }
                }
            } else {
                span { class: "na", "-" }
            }
        }
    }
}
