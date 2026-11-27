// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Natural-product evidence framework: motif counting, ring-family classification,
//! note synthesis, and typed verdict classification.

use crate::model::RingFamily;

pub(crate) mod assessment;
pub(crate) mod chemist;
pub(crate) mod ring_family;
pub(crate) mod verdict;

pub(super) const ERTL_STRONG: f64 = 2.0;
pub(super) const ERTL_MODERATE: f64 = 1.0;
pub(super) const ERTL_AMBIGUOUS: f64 = 0.0;
pub(super) const ERTL_FISHY: f64 = -2.0;

pub(super) const RING_FAMILY_MOTIFS: &[(RingFamily, &[&str])] = &[
    (RingFamily::Steroid, &["steroid"]),
    (RingFamily::Sugar, &["sugar", "tetrahydrofuran"]),
    (
        RingFamily::Macrolide,
        &["macrolide", "macrocycle", "lactone", "lactam"],
    ),
    (RingFamily::Flavonoid, &["flavone", "flavonoid"]),
    (
        RingFamily::Heteroaromatic,
        &[
            "indole",
            "quinoline",
            "isoquinoline",
            "benzofuran",
            "benzothiophene",
            "quinoxaline",
            "purine",
            "chromone",
            "coumarin",
        ],
    ),
];
