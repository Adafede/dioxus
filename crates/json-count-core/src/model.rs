// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the json-count-rs project

//! Column result model, sorting primitives, and progress formatting.

/// Count of non-null values for one top-level JSON column/key.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ColumnResult {
    /// Column / top-level JSON key name.
    pub key: String,
    /// Non-null leaf or row count.
    pub count: u64,
}

/// Column by which results can be sorted.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortColumn {
    /// Lexicographic sort by column name.
    Key,
    /// Numeric sort by non-null count (with key tie-break).
    Count,
}

/// Sort direction.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortDirection {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

impl SortDirection {
    /// Returns the opposite sort direction.
    #[must_use]
    pub const fn toggled(self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }

    /// Returns the ARIA `aria-sort` attribute value.
    #[must_use]
    pub const fn aria_sort(self) -> &'static str {
        match self {
            Self::Asc => "ascending",
            Self::Desc => "descending",
        }
    }

    /// Returns the visual sort direction glyph.
    #[must_use]
    pub const fn indicator(self) -> &'static str {
        match self {
            Self::Asc => " ▲",
            Self::Desc => " ▼",
        }
    }
}

/// Combined active sort column and direction.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SortSpec {
    /// Active sort column.
    pub column: SortColumn,
    /// Active sort direction.
    pub direction: SortDirection,
}

/// Computes the next [`SortSpec`] when `clicked` column header is activated.
#[must_use]
pub const fn next_sort_state(current: Option<SortSpec>, clicked: SortColumn) -> SortSpec {
    match current {
        Some(spec)
            if matches!(
                (spec.column, clicked),
                (SortColumn::Key, SortColumn::Key) | (SortColumn::Count, SortColumn::Count)
            ) =>
        {
            SortSpec {
                column: clicked,
                direction: spec.direction.toggled(),
            }
        }
        _ => {
            let direction = match clicked {
                SortColumn::Key => SortDirection::Asc,
                SortColumn::Count => SortDirection::Desc,
            };
            SortSpec {
                column: clicked,
                direction,
            }
        }
    }
}

/// Returns the ARIA `aria-sort` value for column `col` given `current`.
#[must_use]
pub fn aria_sort_for(current: Option<SortSpec>, col: SortColumn) -> &'static str {
    match current {
        Some(spec) if spec.column == col => spec.direction.aria_sort(),
        _ => "none",
    }
}

/// Returns a sorted copy of `rows` according to `current`.
#[must_use]
pub fn sorted_results(rows: &[ColumnResult], current: Option<SortSpec>) -> Vec<ColumnResult> {
    let mut out = rows.to_vec();
    let Some(spec) = current else {
        return out;
    };
    out.sort_by(|a, b| {
        let ord = match spec.column {
            SortColumn::Key => a.key.cmp(&b.key),
            SortColumn::Count => a.count.cmp(&b.count).then_with(|| a.key.cmp(&b.key)),
        };
        match spec.direction {
            SortDirection::Asc => ord,
            SortDirection::Desc => ord.reverse(),
        }
    });
    out
}

/// Formats a clamped byte-progress line for UI display.
#[must_use]
pub fn progress_message(processed: u64, total: u64) -> String {
    let safe_total = total.max(1);
    let displayed = processed.min(safe_total);
    let percent = (displayed * 100 / safe_total).min(100);
    format!("Scanning {displayed}/{safe_total} bytes ({percent}%)...")
}
