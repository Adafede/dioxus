// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::models::CompoundEntry;
use std::sync::Arc;

#[derive(Clone, PartialEq, Debug)]
pub(in crate::components::results_table) struct PreparedRow {
    pub(super) display_name: Arc<str>,
    pub(super) display_name_short: Arc<str>,
    pub(super) depict_url: Option<Arc<str>>,
    pub(super) doi: Option<Arc<str>>,
    pub(super) statement_id: Option<Arc<str>>,
    pub(super) reference_title_short: Option<Arc<str>>,
    pub(super) short_inchikey: Option<Arc<str>>,
}

pub(in crate::components::results_table) fn prepare_rows(
    rows: &[CompoundEntry],
) -> Arc<[PreparedRow]> {
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
            reference_title_short: entry
                .ref_title
                .as_deref()
                .map(|title| truncate_arc_str(title, 60)),
            short_inchikey: entry.inchikey.as_deref().map(short_inchikey_arc),
            display_name,
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
        assert!(
            prepared
                .depict_url
                .as_deref()
                .is_some_and(|url| url.contains("annotate=cip"))
        );
    }
}
