// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! LIPID MAPS family ranking and name/code mapping.
//!
//! The canonical LIPID MAPS hierarchy order (FA → GL → GP → SP → ST → PR → SL → PK)
//! is used both for sorting the "Filter by chemical family" UI groups and for
//! deriving display codes.

/// LIPID MAPS broad family rank order (FA → GL → GP → SP → ST → PR → SL → PK).
/// Used to sort the "Filter by chemical family" groups in the Results UI so they
/// display in the standard LIPID MAPS classification hierarchy.
pub const LIPID_MAPS_FAMILY_RANK: [(&str, usize); 8] = [
    ("Fatty Acyls", 0),
    ("Glycerolipids", 1),
    ("Glycerophospholipids", 2),
    ("Sphingolipids", 3),
    ("Sterol Lipids", 4),
    ("Prenol Lipids", 5),
    ("Saccharolipids", 6),
    ("Polyketides", 7),
];

/// Lookup helper for [`LIPID_MAPS_FAMILY_RANK`].
pub fn family_rank(family: &str) -> usize {
    LIPID_MAPS_FAMILY_RANK
        .iter()
        .find(|(name, _)| *name == family)
        .map_or(99, |(_, rank)| *rank)
}

/// Strip the trailing `[XX]` code from a category string
/// (e.g. "Fatty Acyls \[FA]" → "Fatty Acyls").
pub fn family_from_category(category: &str) -> &str {
    category.split(" [").next().unwrap_or(category).trim()
}

/// Map a family name to its LIPID MAPS two-letter code.
pub fn family_code(family: &str) -> &str {
    match family {
        "Fatty Acyls" => "FA",
        "Glycerolipids" => "GL",
        "Glycerophospholipids" => "GP",
        "Sphingolipids" => "SP",
        "Sterol Lipids" => "ST",
        "Prenol Lipids" => "PR",
        "Saccharolipids" => "SL",
        "Polyketides" => "PK",
        _ => "-",
    }
}
