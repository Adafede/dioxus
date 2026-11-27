// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Compact fixed-size bitset for LMSD chemical class membership (`<= 128` classes).

/// Membership mask over [`crate::ChemicalClass`] entries keyed by [`crate::ChemicalClass::index`].
///
/// Replaces per-spectrum `HashMap<String, bool>` allocations with a single `Copy`
/// 128-bit integer covering all 86 LMSD main classes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClassMatchSet(u128);

impl ClassMatchSet {
    /// Empty match set.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Full mask covering `count` classes (`0..count`, clamped to `128`).
    #[must_use]
    pub const fn all(count: usize) -> Self {
        if count >= 128 {
            Self(u128::MAX)
        } else if count == 0 {
            Self(0)
        } else {
            Self((1u128 << count) - 1)
        }
    }

    /// Build a set from an iterator of class indices.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn from_indices(indices: impl IntoIterator<Item = usize>) -> Self {
        let mut set = Self::empty();
        for idx in indices {
            set.insert(idx);
        }
        set
    }

    /// Marks class `index` as present (`index < 128`).
    pub const fn insert(&mut self, index: usize) {
        if index < 128 {
            self.0 |= 1u128 << index;
        }
    }

    /// Clears class `index` (`index < 128`).
    pub const fn remove(&mut self, index: usize) {
        if index < 128 {
            self.0 &= !(1u128 << index);
        }
    }

    /// Returns `true` when class `index` is set.
    #[must_use]
    pub const fn contains(self, index: usize) -> bool {
        index < 128 && (self.0 & (1u128 << index)) != 0
    }

    /// Returns `true` when no classes are set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Number of selected/matched classes.
    #[must_use]
    pub const fn len(self) -> usize {
        self.0.count_ones() as usize
    }

    /// Returns `true` when at least one class is shared with `other`.
    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    /// Toggle class `index`.
    pub const fn toggle(&mut self, index: usize) {
        if self.contains(index) {
            self.remove(index);
        } else {
            self.insert(index);
        }
    }
}
