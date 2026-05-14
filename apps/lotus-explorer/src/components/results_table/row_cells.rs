// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::i18n::{
    Locale, TextKey, aria_chemical_structure, aria_search_inchikey, aria_wikidata_entity,
    aria_wikidata_statement, t,
};
use crate::models::CompoundEntry;
use dioxus::prelude::*;
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct RowText {
    pub(super) open_full_size_depiction: &'static str,
    pub(super) open_in_wikidata: &'static str,
    pub(super) open_in_scholia: &'static str,
    pub(super) open_doi: &'static str,
    pub(super) statement: &'static str,
}

pub(super) fn row_text(locale: Locale) -> RowText {
    RowText {
        open_full_size_depiction: t(locale, TextKey::OpenFullSizeDepiction),
        open_in_wikidata: t(locale, TextKey::OpenInWikidata),
        open_in_scholia: t(locale, TextKey::OpenInScholia),
        open_doi: t(locale, TextKey::OpenDoi),
        statement: t(locale, TextKey::Statement),
    }
}

#[component]
pub(super) fn ResultsRowsWindow(
    locale: Locale,
    text: RowText,
    rows: Arc<[CompoundEntry]>,
    order: Arc<[u32]>,
    start_row: usize,
    visible_count: usize,
) -> Element {
    rsx! {
        for i in order.iter().skip(start_row).take(visible_count).copied() {
            {row_view(locale, text, &rows[i as usize], i)}
        }
    }
}

fn row_view(locale: Locale, text: RowText, entry: &CompoundEntry, row_key: u32) -> Element {
    let compound_qid = entry.compound_qid.as_ref();
    let taxon_qid = entry.taxon_qid.as_ref();
    let reference_qid = entry.reference_qid.as_ref();
    let doi = entry.doi();
    let depict_url = entry.depict_url();
    let statement_id = entry.statement_id_str();
    let truncated_ref_title = entry
        .ref_title
        .as_deref()
        .map(|title| truncate_title(title, 55));
    let name: &str = if entry.name.trim().is_empty() {
        entry.compound_qid.as_ref()
    } else {
        &entry.name
    };
    let truncated_compound_name = truncate_title(name, 55);
    rsx! {
        tr { key: "{row_key}", class: "data-row",
            {structure_cell(locale, text, depict_url.as_deref(), name)}
            {compound_cell(locale, text, entry, name, truncated_compound_name, compound_qid)}
            {mass_cell(entry.mass)}
            {formula_cell(entry.formula.as_deref())}
            {taxon_cell(locale, text, entry, taxon_qid)}
            {
                reference_cell(
                    locale,
                    text,
                    entry,
                    reference_qid,
                    doi,
                    statement_id,
                    truncated_ref_title,
                )
            }
            {year_cell(entry.pub_year)}
        }
    }
}

fn structure_cell(locale: Locale, text: RowText, depict_url: Option<&str>, name: &str) -> Element {
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
                span { class: "na", "—" }
            }
        }
    }
}

fn compound_cell(
    locale: Locale,
    text: RowText,
    entry: &CompoundEntry,
    name: &str,
    truncated_name: Cow<'_, str>,
    compound_qid: &str,
) -> Element {
    rsx! {
        td { class: "td-compound",
            div { class: "cell-primary",
                a {
                    href: "https://www.wikidata.org/entity/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "primary-link",
                    title: "{name}",
                    "{truncated_name}"
                }
            }
            div { class: "badge-row",
                a {
                    href: "https://www.wikidata.org/entity/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "id-badge wd",
                    title: "{text.open_in_wikidata}",
                    aria_label: "{aria_wikidata_entity(locale, compound_qid)}",
                    "{compound_qid}"
                }
                a {
                    href: "https://scholia.toolforge.org/chemical/{compound_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "id-badge sc",
                    title: "{text.open_in_scholia}",
                    aria_label: "{text.open_in_scholia}",
                    "Scholia"
                }
                if let Some(ik) = entry.inchikey.as_deref() {
                    a {
                        href: "https://www.wikidata.org/wiki/Special:Search?search={ik}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "id-badge mono inchikey",
                        title: "{ik}",
                        aria_label: "{aria_search_inchikey(locale, ik)}",
                        "{short_inchikey(ik)}"
                    }
                }
            }
        }
    }
}

fn mass_cell(mass: Option<f64>) -> Element {
    rsx! {
        td { class: "td-num",
            if let Some(m) = mass {
                span { "{m:.4}" }
            } else {
                span { class: "na", "—" }
            }
        }
    }
}

fn formula_cell(formula: Option<&str>) -> Element {
    rsx! {
        td { class: "td-formula",
            if let Some(f) = formula {
                span { class: "formula", "{f}" }
            } else {
                span { class: "na", "—" }
            }
        }
    }
}

fn taxon_cell(locale: Locale, text: RowText, entry: &CompoundEntry, taxon_qid: &str) -> Element {
    rsx! {
        td { class: "td-taxon",
            div { class: "cell-primary",
                a {
                    href: "https://www.wikidata.org/entity/{taxon_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "primary-link taxon",
                    "{entry.taxon_name}"
                }
            }
            div { class: "badge-row",
                a {
                    href: "https://www.wikidata.org/entity/{taxon_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "id-badge wd",
                    title: "{text.open_in_wikidata}",
                    aria_label: "{aria_wikidata_entity(locale, taxon_qid)}",
                    "{taxon_qid}"
                }
            }
        }
    }
}

fn reference_cell(
    locale: Locale,
    text: RowText,
    entry: &CompoundEntry,
    reference_qid: &str,
    doi: Option<&str>,
    statement_id: Option<&str>,
    truncated_ref_title: Option<Cow<'_, str>>,
) -> Element {
    rsx! {
        td { class: "td-ref",
            div { class: "cell-primary",
                if let (Some(full_title), Some(display_title)) = (
                    entry.ref_title.as_deref(),
                    truncated_ref_title.as_deref(),
                )
                {
                    a {
                        href: "https://www.wikidata.org/entity/{reference_qid}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "primary-link",
                        title: "{full_title}",
                        "{display_title}"
                    }
                } else {
                    a {
                        href: "https://www.wikidata.org/entity/{reference_qid}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "primary-link",
                        "{reference_qid}"
                    }
                }
            }
            div { class: "badge-row",
                a {
                    href: "https://www.wikidata.org/entity/{reference_qid}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "id-badge wd",
                    title: "{text.open_in_wikidata}",
                    aria_label: "{aria_wikidata_entity(locale, reference_qid)}",
                    "{reference_qid}"
                }
                if let Some(d) = doi {
                    a {
                        href: "https://doi.org/{d}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "id-badge doi",
                        title: "{text.open_doi}",
                        aria_label: "{text.open_doi}",
                        "DOI"
                    }
                }
                if let Some(stmt) = statement_id {
                    a {
                        href: "https://www.wikidata.org/entity/statement/{stmt}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "id-badge stmt mono",
                        title: "{stmt}",
                        aria_label: "{aria_wikidata_statement(locale, stmt)}",
                        "{text.statement}"
                    }
                }
            }
        }
    }
}

fn year_cell(pub_year: Option<i16>) -> Element {
    rsx! {
        td { class: "td-year",
            if let Some(y) = pub_year {
                span { "{y}" }
            } else {
                span { class: "na", "—" }
            }
        }
    }
}

fn short_inchikey(ik: &str) -> &str {
    ik.split('-').next().unwrap_or(ik)
}

fn truncate_title(title: &str, max_chars: usize) -> Cow<'_, str> {
    let trimmed = title.trim();
    if trimmed.chars().count() <= max_chars {
        return Cow::Borrowed(trimmed);
    }
    let mut out: String = trimmed.chars().take(max_chars).collect();
    out.push('…');
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_inchikey_returns_first_segment() {
        assert_eq!(short_inchikey("AAAA-BBBB-CCCC"), "AAAA");
        assert_eq!(short_inchikey("NOSPLIT"), "NOSPLIT");
    }

    #[test]
    fn truncate_title_borrows_when_already_short() {
        let title = "Short title";
        let truncated = truncate_title(title, 55);
        assert!(matches!(truncated, Cow::Borrowed(_)));
        assert_eq!(truncated, "Short title");
    }

    #[test]
    fn truncate_title_trims_and_appends_ellipsis() {
        let truncated = truncate_title("  This title is definitely longer than ten chars  ", 10);
        assert!(matches!(truncated, Cow::Owned(_)));
        assert_eq!(truncated, "This title…");
    }
}
