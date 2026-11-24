// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

use crate::{
    AdductFamily, ErrorBucket, ErrorMeasurement, MedianTracker, OrderedF64, PrecursorStats,
    ReservoirSampler,
};

#[test]
fn adduct_family_covers_all_categories() {
    assert_eq!(AdductFamily::from_label("[M+H]+"), AdductFamily::Protonated);
    assert_eq!(
        AdductFamily::from_label(" [m + h] + "),
        AdductFamily::Protonated
    );
    assert_eq!(
        AdductFamily::from_label("[M-H]-"),
        AdductFamily::Deprotonated
    );
    assert_eq!(
        AdductFamily::from_label("[M+NH4]+"),
        AdductFamily::AlkaliAmmonium
    );
    assert_eq!(
        AdductFamily::from_label("[M+Na]+"),
        AdductFamily::AlkaliAmmonium
    );
    assert_eq!(
        AdductFamily::from_label("[M+Mg]2+"),
        AdductFamily::MetalComplex
    );
    assert_eq!(
        AdductFamily::from_label("[M+Ca]2+"),
        AdductFamily::MetalComplex
    );
    assert_eq!(
        AdductFamily::from_label("[M+Fe]2+"),
        AdductFamily::MetalComplex
    );
    assert_eq!(AdductFamily::from_label("[M+Cl]-"), AdductFamily::Halide);
    assert_eq!(AdductFamily::from_label("[M+Br]-"), AdductFamily::Halide);
    assert_eq!(AdductFamily::from_label("[M+CH3COO]-"), AdductFamily::Other);
}

#[test]
fn median_tracker_and_ordered_f64_handle_nan_and_streams() {
    let mut tracker = MedianTracker::default();
    assert_eq!(tracker.median(), 0.0);
    tracker.push(f64::NAN);
    assert_eq!(tracker.median(), 0.0);
    tracker.push(10.0);
    assert_eq!(tracker.median(), 10.0);
    tracker.push(2.0);
    assert_eq!(tracker.median(), 6.0);
    tracker.push(7.0);
    assert_eq!(tracker.median(), 7.0);

    // Total order equality must agree with Ord even on NaN and signed zero.
    assert_eq!(OrderedF64(f64::NAN), OrderedF64(f64::NAN));
    assert_ne!(OrderedF64(-0.0), OrderedF64(0.0));
}

#[test]
fn record_error_updates_buckets_moments_median_and_sorting_helpers() {
    let mut stats = PrecursorStats::default();
    let samples = [
        (0.000_05, 0.25, AdductFamily::Protonated, 200.0, None, None),
        (
            0.000_3,
            0.8,
            AdductFamily::Deprotonated,
            300.0,
            Some("C"),
            Some("CH4"),
        ),
        (
            0.000_8,
            3.0,
            AdductFamily::AlkaliAmmonium,
            400.0,
            Some("CC"),
            Some("C2H6"),
        ),
        (
            0.003_0,
            8.0,
            AdductFamily::MetalComplex,
            500.0,
            Some("CCC"),
            Some("C3H8"),
        ),
        (
            0.020_0,
            25.0,
            AdductFamily::Halide,
            800.0,
            Some("CCCC"),
            Some("C4H10"),
        ),
    ];
    for (abs_da, abs_ppm, family, pepmass, smiles, formula) in samples {
        stats.record_error(ErrorMeasurement {
            abs_error_da: abs_da,
            abs_ppm,
            adduct_family: family,
            ppm_error: abs_ppm,
            signed_error_da: abs_da,
            pepmass_header: pepmass,
            ms2_precursor_peak: Some(pepmass),
            smiles,
            calculated_mass: Some(pepmass - 1.0),
            expected_mass: Some(pepmass - abs_da),
            formula,
        });
    }
    assert_eq!(stats.spectra, 5);
    assert_eq!(stats.cumulative_bucket_count(ErrorBucket::Within0_1Mda), 1);
    assert_eq!(stats.cumulative_bucket_count(ErrorBucket::Within0_5Mda), 2);
    assert_eq!(stats.cumulative_bucket_count(ErrorBucket::Within1Mda), 3);
    assert_eq!(stats.cumulative_bucket_count(ErrorBucket::Within5Mda), 4);
    assert_eq!(stats.cumulative_bucket_count(ErrorBucket::Above5Mda), 1);
    assert_eq!(stats.cumulative_bucket_count(ErrorBucket::Within10Ppm), 4);
    assert_eq!(stats.cumulative_bucket_count(ErrorBucket::Above10Ppm), 1);
    assert_eq!(stats.observed_precursor_min, 200.0);
    assert_eq!(stats.observed_precursor_max, 800.0);
    assert_eq!(stats.observed_precursor_median, 400.0);
    assert_eq!(stats.sorted_high_error_smiles().len(), 1);
    assert_eq!(stats.sorted_high_error_smiles()[0].0, "CCCC");

    let mut reservoir = ReservoirSampler::new(3);
    for i in 0..10 {
        reservoir.push(i);
    }
    assert_eq!(reservoir.seen(), 10);
    assert_eq!(reservoir.items().len(), 3);
}
