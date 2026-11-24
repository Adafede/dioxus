// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

use crate::{
    ColumnResult, SortColumn, SortDirection, SortSpec, aria_sort_for, next_sort_state,
    sorted_results,
};

#[test]
fn sort_state_transitions_and_aria_labels() {
    assert_eq!(
        aria_sort_for(None, SortColumn::Key),
        "none",
        "inactive sort reports none"
    );
    let spec1 = next_sort_state(None, SortColumn::Key);
    assert_eq!(
        spec1,
        SortSpec {
            column: SortColumn::Key,
            direction: SortDirection::Asc
        },
        "first key click sorts ascending"
    );
    assert_eq!(
        aria_sort_for(Some(spec1), SortColumn::Key),
        "ascending",
        "active key reports ascending"
    );
    assert_eq!(
        aria_sort_for(Some(spec1), SortColumn::Count),
        "none",
        "other column reports none"
    );
    assert_eq!(spec1.direction.indicator(), " ▲", "asc indicator");

    let spec2 = next_sort_state(Some(spec1), SortColumn::Key);
    assert_eq!(
        spec2,
        SortSpec {
            column: SortColumn::Key,
            direction: SortDirection::Desc
        },
        "second key click toggles descending"
    );
    assert_eq!(
        aria_sort_for(Some(spec2), SortColumn::Key),
        "descending",
        "descending aria label"
    );
    assert_eq!(spec2.direction.indicator(), " ▼", "desc indicator");

    let spec3 = next_sort_state(Some(spec2), SortColumn::Count);
    assert_eq!(
        spec3,
        SortSpec {
            column: SortColumn::Count,
            direction: SortDirection::Desc
        },
        "first count click defaults to descending"
    );
    let spec4 = next_sort_state(Some(spec3), SortColumn::Count);
    assert_eq!(
        spec4,
        SortSpec {
            column: SortColumn::Count,
            direction: SortDirection::Asc
        },
        "second count click toggles ascending"
    );
}

#[test]
fn sorted_results_orders_by_selected_column_and_direction() {
    let rows = vec![
        ColumnResult {
            key: "zeta".to_string(),
            count: 2,
        },
        ColumnResult {
            key: "alpha".to_string(),
            count: 10,
        },
        ColumnResult {
            key: "beta".to_string(),
            count: 2,
        },
    ];
    assert_eq!(
        sorted_results(&rows, None),
        rows,
        "unsorted preserves order"
    );

    let key_asc = sorted_results(
        &rows,
        Some(SortSpec {
            column: SortColumn::Key,
            direction: SortDirection::Asc,
        }),
    );
    assert_eq!(
        key_asc.iter().map(|r| r.key.as_str()).collect::<Vec<_>>(),
        vec!["alpha", "beta", "zeta"],
        "key asc"
    );

    let key_desc = sorted_results(
        &rows,
        Some(SortSpec {
            column: SortColumn::Key,
            direction: SortDirection::Desc,
        }),
    );
    assert_eq!(
        key_desc.iter().map(|r| r.key.as_str()).collect::<Vec<_>>(),
        vec!["zeta", "beta", "alpha"],
        "key desc"
    );

    let count_desc = sorted_results(
        &rows,
        Some(SortSpec {
            column: SortColumn::Count,
            direction: SortDirection::Desc,
        }),
    );
    assert_eq!(
        count_desc
            .iter()
            .map(|r| (r.key.as_str(), r.count))
            .collect::<Vec<_>>(),
        vec![("alpha", 10), ("zeta", 2), ("beta", 2)],
        "count desc with key tiebreak"
    );

    let count_asc = sorted_results(
        &rows,
        Some(SortSpec {
            column: SortColumn::Count,
            direction: SortDirection::Asc,
        }),
    );
    assert_eq!(
        count_asc
            .iter()
            .map(|r| (r.key.as_str(), r.count))
            .collect::<Vec<_>>(),
        vec![("beta", 2), ("zeta", 2), ("alpha", 10)],
        "count asc with key tiebreak"
    );
}
