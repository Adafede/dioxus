// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::i18n::{
    Locale, TextKey, aria_chemical_structure, aria_search_inchikey, aria_wikidata_entity,
    aria_wikidata_statement, t,
};
use crate::models::CompoundEntry;
use dioxus::prelude::*;
use std::sync::Arc;

#[derive(Clone, PartialEq)]
pub(super) struct PreparedRow {
    pub(super) display_name: Arc<str>,
    pub(super) display_name_short: Arc<str>,
    pub(super) depict_url: Option<Arc<str>>,
    pub(super) doi: Option<Arc<str>>,
    pub(super) statement_id: Option<Arc<str>>,
    pub(super) reference_title_short: Option<Arc<str>>,
    pub(super) short_inchikey: Option<Arc<str>>,
}

pub(super) fn prepare_rows(rows: &[CompoundEntry]) -> Arc<[PreparedRow]> {
    let prepared: Vec<PreparedRow> = rows.iter().map(PreparedRow::from_entry).collect();
    Arc::from(prepared.into_boxed_slice())
}

impl PreparedRow {
    fn from_entry(entry: &CompoundEntry) -> Self {
        let display_name = normalized_display_name(entry);
        Self {
            display_name_short: truncate_arc_str(&display_name, 60),
            depict_url: depict_url_cached(entry),
            doi: trimmed_optional_arc(entry.ref_doi.as_deref()),
            statement_id: trimmed_statement_id_arc(entry.statement.as_deref()),
            reference_title_short: entry.ref_title.as_deref().map(|title| truncate_arc_str(title, 60)),
            short_inchikey: entry.inchikey.as_deref().map(short_inchikey_arc),
            display_name,
        }
    }
}

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
    prepared_rows: Arc<[PreparedRow]>,
    order: Arc<[u32]>,
    start_row: usize,
    visible_count: usize,
) -> Element {
    rsx! {
        for i in order.iter().skip(start_row).take(visible_count).copied() {
            {row_view(locale, text, &rows[i as usize], &prepared_rows[i as usize], i)}
        }
    }
}

fn row_view(
    locale: Locale,
    text: RowText,
    entry: &CompoundEntry,
    prepared: &PreparedRow,
    row_key: u32,
) -> Element {
    let compound_qid = entry.compound_qid.as_ref();
    let taxon_qid = entry.taxon_qid.as_ref();
    let reference_qid = entry.reference_qid.as_ref();
    let doi = prepared.doi.as_deref();
    let statement_id = prepared.statement_id.as_deref();
    let name = prepared.display_name.as_ref();
    rsx! {
        tr { key: "{row_key}", class: "data-row",
            {structure_cell(locale, text, prepared.depict_url.as_deref(), name)}
            {
                compound_cell(
                    locale,
                    text,
                    entry,
                    prepared,
                    name,
                    compound_qid,
                )
            }
            {mass_cell(entry.mass)}
            {formula_cell(entry.formula.as_deref())}
            {taxon_cell(locale, text, entry, taxon_qid)}
            {
                reference_cell(
                    locale,
                    text,
                    entry,
                    prepared,
                    reference_qid,
                    doi,
                    statement_id,
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
    prepared: &PreparedRow,
    name: &str,
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
                    "{prepared.display_name_short}"
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
                        "{prepared.short_inchikey.as_deref().unwrap_or(ik)}"
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
    prepared: &PreparedRow,
    reference_qid: &str,
    doi: Option<&str>,
    statement_id: Option<&str>,
) -> Element {
    rsx! {
        td { class: "td-ref",
            div { class: "cell-primary",
                if let (Some(full_title), Some(display_title)) = (
                    entry.ref_title.as_deref(),
                    prepared.reference_title_short.as_deref(),
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

fn short_inchikey_arc(ik: &str) -> Arc<str> {
    Arc::<str>::from(ik.split('-').next().unwrap_or(ik))
}

fn truncate_arc_str(title: &str, max_chars: usize) -> Arc<str> {
    let trimmed = title.trim();
    if trimmed.chars().count() <= max_chars {
        return Arc::<str>::from(trimmed);
    }
    let mut out: String = trimmed.chars().take(max_chars).collect();
    out.push('…');
    Arc::<str>::from(out)
}

fn normalized_display_name(entry: &CompoundEntry) -> Arc<str> {
    let trimmed = entry.name.trim();
    if trimmed.is_empty() {
        entry.compound_qid.clone()
    } else if trimmed.len() == entry.name.len() {
        entry.name.clone()
    } else {
        Arc::<str>::from(trimmed)
    }
}

fn depict_url_cached(entry: &CompoundEntry) -> Option<Arc<str>> {
    let smiles = entry.smiles.as_deref()?.trim();
    if smiles.is_empty() || smiles.contains('\n') {
        return None;
    }
    Some(Arc::<str>::from(format!(
        "https://www.simolecule.com/cdkdepict/depict/cow/svg?smi={}&annotate=cip",
        urlencoding::encode(smiles)
    )))
}

fn trimmed_optional_arc(value: Option<&str>) -> Option<Arc<str>> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(Arc::<str>::from(trimmed))
    }
}

fn trimmed_statement_id_arc(value: Option<&str>) -> Option<Arc<str>> {
    const STMT_PREFIX: &str = "http://www.wikidata.org/entity/statement/";
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(Arc::<str>::from(
        trimmed.strip_prefix(STMT_PREFIX).unwrap_or(trimmed),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_inchikey_returns_first_segment() {
        assert_eq!(short_inchikey_arc("AAAA-BBBB-CCCC").as_ref(), "AAAA");
        assert_eq!(short_inchikey_arc("NOSPLIT").as_ref(), "NOSPLIT");
    }

    #[test]
    fn truncate_title_borrows_when_already_short() {
        let title = "Short title";
        let truncated = truncate_arc_str(title, 60);
        assert_eq!(truncated.as_ref(), "Short title");
    }

    #[test]
    fn truncate_title_trims_and_appends_ellipsis() {
        let truncated = truncate_arc_str("  This title is definitely longer than ten chars  ", 10);
        assert_eq!(truncated.as_ref(), "This title…");
    }

    #[test]
    fn prepared_row_caches_trimmed_and_derived_fields() {
        let entry = CompoundEntry {
            compound_qid: Arc::<str>::from("Q1"),
            name: Arc::<str>::from("  Alpha  "),
            inchikey: Some(Arc::<str>::from("AAAA-BBBB-CCCC")),
            smiles: Some(Arc::<str>::from("CCO")),
            mass: None,
            formula: None,
            taxon_qid: Arc::<str>::from("T1"),
            taxon_name: Arc::<str>::from("Taxon"),
            reference_qid: Arc::<str>::from("R1"),
            ref_title: Some(Arc::<str>::from("  A fairly short title  ")),
            ref_doi: Some(Arc::<str>::from(" 10.1000/test ")),
            pub_year: None,
            statement: Some(Arc::<str>::from(
                "http://www.wikidata.org/entity/statement/Q1-ABC",
            )),
        };

        let prepared = PreparedRow::from_entry(&entry);
        assert_eq!(prepared.display_name.as_ref(), "Alpha");
        assert_eq!(prepared.display_name_short.as_ref(), "Alpha");
        assert_eq!(prepared.short_inchikey.as_deref(), Some("AAAA"));
        assert_eq!(prepared.doi.as_deref(), Some("10.1000/test"));
        assert_eq!(prepared.statement_id.as_deref(), Some("Q1-ABC"));
        assert!(prepared.depict_url.as_deref().is_some_and(|url| url.contains("annotate=cip")));
    }
}
