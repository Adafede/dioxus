// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use crate::models::{CompoundEntry, SortColumn, SortDir, SortState};
use std::cmp::Ordering;
use std::sync::Arc;

#[must_use]
pub(super) fn build_sorted_indices(rows: &[CompoundEntry], sort: SortState) -> Arc<[u32]> {
    let mut idx: Vec<u32> = (0..rows.len() as u32).collect();
    idx.sort_by(|&a, &b| compare_entries(&rows[a as usize], &rows[b as usize], sort));
    Arc::from(idx.into_boxed_slice())
}

fn compare_entries(a: &CompoundEntry, b: &CompoundEntry, sort: SortState) -> Ordering {
    let cmp = match sort.col {
        SortColumn::Name => a.name.cmp(&b.name),
        SortColumn::Mass => a.mass.partial_cmp(&b.mass).unwrap_or(Ordering::Equal),
        SortColumn::Formula => a.formula.cmp(&b.formula),
        SortColumn::TaxonName => a.taxon_name.cmp(&b.taxon_name),
        SortColumn::PubYear => a.pub_year.cmp(&b.pub_year),
        SortColumn::RefTitle => a.ref_title.cmp(&b.ref_title),
    };

    if sort.dir == SortDir::Desc {
        cmp.reverse()
    } else {
        cmp
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
}
