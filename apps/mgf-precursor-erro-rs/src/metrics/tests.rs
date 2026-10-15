// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

// Host tests for `merge` and `stats`.
//
// `merge` and `stats` used to be `#[cfg(target_arch = "wasm32")]`, so `cargo
// test` on any other target compiled none of them. That is where three of the
// worst defects in this crate lived, invisible because nothing reached them.
use super::*;

/// A single-spectrum accumulator.
///
/// `abs_error_da_mean`/`_rms` are not running sums written by this module:
/// `parser::block` writes one block's mean as its own error and `merge`
/// reweights it by spectrum count, so draining the field is what `merge` sums.
fn stats_with_errors(da: &[f64], ppm: &[f64]) -> PrecursorStats {
    let mut s = PrecursorStats::default();
    for (&da_err, &ppm_err) in da.iter().zip(ppm) {
        s.da_error_histogram.add_value(da_err);
        s.ppm_error_histogram.add_value(ppm_err);
        PrecursorStats::push_sampled_value(
            da_err,
            &mut s.absolute_error_da_values,
            &mut s.absolute_error_da_sample_seen,
        );
        PrecursorStats::push_sampled_value(
            ppm_err,
            &mut s.absolute_error_ppm_values,
            &mut s.absolute_error_ppm_sample_seen,
        );
    }
    let mean = da.iter().sum::<f64>() / usize_to_f64(da.len());
    s.abs_error_da_mean = mean;
    s.abs_error_da_rms = da.iter().map(|v| v * v).sum::<f64>().sqrt();
    s.spectra = 1;
    s.total_spectra = 1;
    s
}

/// A block that produced no reference mass.
///
/// `parser::block`'s skipped-spectrum path does exactly this: `total_spectra: 1`,
// `skipped_spectra: 1`, and `spectra` left at the `Default` `0`. It is the
/// shape the scanner merges into its running accumulator.
fn skipped_block() -> PrecursorStats {
    PrecursorStats {
        total_spectra: 1,
        skipped_spectra: 1,
        ..PrecursorStats::default()
    }
}

/// `merge_precursor_scalar_stats` divides by `total_spectra`. A block whose
/// spectrum was skipped carries `spectra = 0`, so merging one *first* made the
/// divisor `0.0` and the mean became `NaN` — and `NaN` propagates through every
/// later merge, so a single skipped block poisoned a whole multi-file summary.
#[test]
fn merge_does_not_produce_nan_when_a_block_had_zero_spectra() {
    let zero = PrecursorStats::default();
    let good = stats_with_errors(&[1.0, 2.0], &[3.0, 4.0]);

    let merged = merge_precursor_stats(zero.clone(), &skipped_block());
    assert!(
        !merged.abs_error_da_mean.is_nan(),
        "a skipped block merged first produced NaN, which would then survive every later merge"
    );

    let merged = merge_precursor_stats(good, &skipped_block());
    assert!(
        !merged.abs_error_da_mean.is_nan(),
        "a skipped block merged last produced NaN"
    );

    let merged = merge_precursor_stats(stats_with_errors(&[1.0], &[1.0]), &zero);
    assert!(
        !merged.abs_error_da_mean.is_nan(),
        "an accumulator with no spectra at all produced NaN"
    );

    assert!(
        !merged.abs_error_da_mean.is_infinite(),
        "an infinite mean means the divisor was zero and the quotient was not guarded"
    );
}

/// Accumulators merged in either order must agree, and must not equal the empty
/// accumulator. Order-independence is the property the two-file merge UI relies
/// on.
#[test]
fn merge_is_commutative_for_identical_inputs() {
    let a = stats_with_errors(&[1.0], &[10.0]);
    let b = stats_with_errors(&[3.0], &[30.0]);

    let a_then_b = merge_precursor_stats(a.clone(), &b);
    let b_then_a = merge_precursor_stats(b, &a);

    assert_eq!(
        a_then_b.abs_error_da_mean, 2.0,
        "the pooled mean of two single-spectrum blocks must be their spectrum-count-weighted average"
    );
    assert_eq!(
        b_then_a.abs_error_da_mean, 2.0,
        "the pooled mean depends on the order of the merge"
    );
    assert_eq!(a_then_b.abs_error_da_mean, b_then_a.abs_error_da_mean);
    assert_eq!(a_then_b.abs_error_da_rms, b_then_a.abs_error_da_rms);
    assert_ne!(a_then_b.abs_error_da_mean, 0.0);
}
