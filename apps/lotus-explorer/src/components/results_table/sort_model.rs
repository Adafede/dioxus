// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::models::{CompoundEntry, SortColumn, SortDir, SortState};
use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Clone, PartialEq, Debug)]
pub(super) struct SortIndexCache {
    by_name: Arc<[u32]>,
    by_mass: Arc<[u32]>,
    by_formula: Arc<[u32]>,
    by_taxon_name: Arc<[u32]>,
    by_pub_year: Arc<[u32]>,
    by_ref_title: Arc<[u32]>,
}

impl SortIndexCache {
    fn ascending_indices(&self, col: SortColumn) -> &Arc<[u32]> {
        match col {
            SortColumn::Name => &self.by_name,
            SortColumn::Mass => &self.by_mass,
            SortColumn::Formula => &self.by_formula,
            SortColumn::TaxonName => &self.by_taxon_name,
            SortColumn::PubYear => &self.by_pub_year,
            SortColumn::RefTitle => &self.by_ref_title,
        }
    }
}

#[must_use]
pub(super) fn build_sort_index_cache(rows: &[CompoundEntry]) -> SortIndexCache {
    SortIndexCache {
        by_name: build_sorted_indices_for_column(rows, SortColumn::Name),
        by_mass: build_sorted_indices_for_column(rows, SortColumn::Mass),
        by_formula: build_sorted_indices_for_column(rows, SortColumn::Formula),
        by_taxon_name: build_sorted_indices_for_column(rows, SortColumn::TaxonName),
        by_pub_year: build_sorted_indices_for_column(rows, SortColumn::PubYear),
        by_ref_title: build_sorted_indices_for_column(rows, SortColumn::RefTitle),
    }
}

#[must_use]
pub(super) fn indices_for_sort(cache: &SortIndexCache, sort: SortState) -> Arc<[u32]> {
    let ascending = cache.ascending_indices(sort.col);
    if sort.dir == SortDir::Asc {
        ascending.clone()
    } else {
        reversed_indices(ascending)
    }
}

#[cfg(test)]
#[must_use]
pub(super) fn build_sorted_indices(rows: &[CompoundEntry], sort: SortState) -> Arc<[u32]> {
    let ascending = build_sorted_indices_for_column(rows, sort.col);
    if sort.dir == SortDir::Asc {
        ascending
    } else {
        reversed_indices(&ascending)
    }
}

fn build_sorted_indices_for_column(rows: &[CompoundEntry], column: SortColumn) -> Arc<[u32]> {
    let mut idx: Vec<u32> = (0..rows.len() as u32).collect();
    idx.sort_by(|&a, &b| {
        compare_entries(&rows[a as usize], &rows[b as usize], column).then_with(|| a.cmp(&b))
    });
    Arc::from(idx.into_boxed_slice())
}

fn reversed_indices(indices: &[u32]) -> Arc<[u32]> {
    let mut reversed = Vec::with_capacity(indices.len());
    reversed.extend(indices.iter().rev().copied());
    Arc::from(reversed.into_boxed_slice())
}

fn compare_entries(a: &CompoundEntry, b: &CompoundEntry, column: SortColumn) -> Ordering {
    match column {
        SortColumn::Name => a.name.cmp(&b.name),
        SortColumn::Mass => a.mass.partial_cmp(&b.mass).unwrap_or(Ordering::Equal),
        SortColumn::Formula => a.formula.cmp(&b.formula),
        SortColumn::TaxonName => a.taxon_name.cmp(&b.taxon_name),
        SortColumn::PubYear => a.pub_year.cmp(&b.pub_year),
        SortColumn::RefTitle => a.ref_title.cmp(&b.ref_title),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn entry(
        name: &str,
        mass: Option<f64>,
        formula: Option<&str>,
        taxon_name: &str,
        pub_year: Option<i16>,
        ref_title: Option<&str>,
    ) -> CompoundEntry {
        CompoundEntry {
            compound_qid: Arc::<str>::from(format!("Q-{name}")),
            name: Arc::<str>::from(name),
            inchikey: None,
            smiles: None,
            mass,
            formula: formula.map(Arc::<str>::from),
            taxon_qid: Arc::<str>::from(format!("T-{taxon_name}")),
            taxon_name: Arc::<str>::from(taxon_name),
            reference_qid: Arc::<str>::from("R-1"),
            ref_title: ref_title.map(Arc::<str>::from),
            ref_doi: None,
            pub_year,
            statement: None,
        }
    }

    #[test]
    fn sorts_by_name_ascending_by_default() {
        let rows = vec![
            entry(
                "Gamma",
                Some(3.0),
                Some("C3"),
                "Taxon C",
                Some(2003),
                Some("Ref C"),
            ),
            entry(
                "Alpha",
                Some(1.0),
                Some("C1"),
                "Taxon A",
                Some(2001),
                Some("Ref A"),
            ),
            entry(
                "Beta",
                Some(2.0),
                Some("C2"),
                "Taxon B",
                Some(2002),
                Some("Ref B"),
            ),
        ];

        let order = build_sorted_indices(&rows, SortState::default());
        assert_eq!(order.as_ref(), &[1, 2, 0]);
    }

    #[test]
    fn sorts_by_mass_descending() {
        let rows = vec![
            entry(
                "Alpha",
                Some(10.0),
                Some("C1"),
                "Taxon A",
                Some(2001),
                Some("Ref A"),
            ),
            entry(
                "Beta",
                Some(30.0),
                Some("C2"),
                "Taxon B",
                Some(2002),
                Some("Ref B"),
            ),
            entry(
                "Gamma",
                Some(20.0),
                Some("C3"),
                "Taxon C",
                Some(2003),
                Some("Ref C"),
            ),
        ];

        let order = build_sorted_indices(
            &rows,
            SortState {
                col: SortColumn::Mass,
                dir: SortDir::Desc,
            },
        );
        assert_eq!(order.as_ref(), &[1, 2, 0]);
    }

    #[test]
    fn sorts_optional_reference_titles() {
        let rows = vec![
            entry(
                "Alpha",
                Some(10.0),
                Some("C1"),
                "Taxon A",
                Some(2001),
                Some("Zeta"),
            ),
            entry("Beta", Some(30.0), Some("C2"), "Taxon B", Some(2002), None),
            entry(
                "Gamma",
                Some(20.0),
                Some("C3"),
                "Taxon C",
                Some(2003),
                Some("Alpha"),
            ),
        ];

        let order = build_sorted_indices(
            &rows,
            SortState {
                col: SortColumn::RefTitle,
                dir: SortDir::Asc,
            },
        );
        assert_eq!(order.as_ref(), &[1, 2, 0]);
    }

    #[test]
    fn cache_returns_same_indices_as_direct_sort_for_asc_and_desc() {
        let rows = vec![
            entry(
                "Gamma",
                Some(3.0),
                Some("C3"),
                "Taxon C",
                Some(2003),
                Some("Ref C"),
            ),
            entry(
                "Alpha",
                Some(1.0),
                Some("C1"),
                "Taxon A",
                Some(2001),
                Some("Ref A"),
            ),
            entry(
                "Beta",
                Some(2.0),
                Some("C2"),
                "Taxon B",
                Some(2002),
                Some("Ref B"),
            ),
        ];

        let cache = build_sort_index_cache(&rows);
        let asc = indices_for_sort(
            &cache,
            SortState {
                col: SortColumn::Name,
                dir: SortDir::Asc,
            },
        );
        let desc = indices_for_sort(
            &cache,
            SortState {
                col: SortColumn::Name,
                dir: SortDir::Desc,
            },
        );

        assert_eq!(
            asc.as_ref(),
            build_sorted_indices(&rows, SortState::default()).as_ref()
        );
        assert_eq!(desc.as_ref(), &[0, 2, 1]);
    }

    #[test]
    fn descending_sort_is_exact_reverse_of_ascending_order() {
        let rows = vec![
            entry(
                "Alpha",
                Some(10.0),
                Some("C1"),
                "Taxon A",
                Some(2001),
                Some("Zeta"),
            ),
            entry(
                "Alpha",
                Some(10.0),
                Some("C1"),
                "Taxon A",
                Some(2001),
                Some("Alpha"),
            ),
            entry(
                "Gamma",
                Some(20.0),
                Some("C3"),
                "Taxon C",
                Some(2003),
                Some("Beta"),
            ),
        ];

        let asc = build_sorted_indices(
            &rows,
            SortState {
                col: SortColumn::RefTitle,
                dir: SortDir::Asc,
            },
        );
        let desc = build_sorted_indices(
            &rows,
            SortState {
                col: SortColumn::RefTitle,
                dir: SortDir::Desc,
            },
        );

        let expected_desc: Vec<u32> = asc.iter().rev().copied().collect();
        assert_eq!(desc.as_ref(), expected_desc.as_slice());
    }
}
