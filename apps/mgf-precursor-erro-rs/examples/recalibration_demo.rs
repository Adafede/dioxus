// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

// Example: Using Precursor-Driven MS2 Fragment Recalibration
//
// This example demonstrates how to use the recalibration module to:
// 1. Define a calibration model
// 2. Recalibrate fragments
// 3. Track diagnostic metrics
// 4. Generate visualization plots

use mgf_analysis_core::{CalibrationModel, Peak, RecalibrationStats, recalibrate_fragments};
use mgf_precursor_erro_rs::plotting::{
    render_recalibration_diagnostic_histogram, render_recalibration_diagnostic_ppm,
};

fn print_fragments(title: &str, fragments: &[Peak], include_intensity: bool) {
    println!("{title}");
    for (i, peak) in fragments.iter().enumerate() {
        if include_intensity {
            println!(
                "  Fragment {i}: m/z = {mz:.6}, intensity = {intensity}",
                mz = peak.mz,
                intensity = peak.intensity,
            );
        } else {
            println!("  Fragment {i}: m/z = {mz:.6}", mz = peak.mz);
        }
    }
}

fn run_tof_examples(precursor_ms1: f64, precursor_ms2: f64) {
    println!("\n=== Example 1: TOF Recalibration (Full Correction) ===");

    let mut fragments = vec![
        Peak::new(100.0, 5000.0),
        Peak::new(250.0, 3000.0),
        Peak::new(400.0, 1000.0),
    ];
    print_fragments("Before recalibration:", &fragments, true);

    recalibrate_fragments(
        &mut fragments,
        precursor_ms1,
        precursor_ms2,
        CalibrationModel::tof(1.0),
    );
    print_fragments("\nAfter recalibration (λ = 1.0):", &fragments, true);

    println!("\n=== Example 2: TOF Recalibration (Partial Correction) ===");
    let mut fragments = vec![
        Peak::new(100.0, 5000.0),
        Peak::new(250.0, 3000.0),
        Peak::new(400.0, 1000.0),
    ];
    recalibrate_fragments(
        &mut fragments,
        precursor_ms1,
        precursor_ms2,
        CalibrationModel::tof(0.5),
    );
    print_fragments("After recalibration (λ = 0.5):", &fragments, true);
}

fn run_orbitrap_example() {
    println!("\n=== Example 3: Orbitrap Recalibration (ppm-based) ===");

    let mut fragments = vec![Peak::new(200.0, 5000.0), Peak::new(400.0, 3000.0)];
    let precursor_ms1 = 500.0000;
    let precursor_ms2 = 500.0050;

    println!("Precursor error: ~10 ppm");
    print_fragments("Before recalibration:", &fragments, false);

    recalibrate_fragments(
        &mut fragments,
        precursor_ms1,
        precursor_ms2,
        CalibrationModel::orbitrap(1.0),
    );

    print_fragments("\nAfter recalibration (λ = 1.0):", &fragments, false);
}

fn run_diagnostics_example() -> RecalibrationStats {
    println!("\n=== Example 4: Diagnostic Metrics Tracking ===");

    let mut diagnostics = RecalibrationStats::new();
    // Consistent errors at m/z = 500: error_da = error_ppm * 500e-6.
    let errors = [
        (10.5, 5.0, 0.005_25, 0.002_50, "protonated"),
        (12.0, 6.0, 0.006_00, 0.003_00, "protonated"),
        (-8.5, -4.0, -0.004_25, -0.002_00, "deprotonated"),
        (9.5, 4.5, 0.004_75, 0.002_25, "protonated"),
        (11.0, 5.5, 0.005_50, 0.002_75, "deprotonated"),
    ];

    for (error_ppm_before, error_ppm_after, error_da_before, error_da_after, family) in errors {
        diagnostics.push_error(
            error_ppm_before,
            error_ppm_after,
            error_da_before,
            error_da_after,
            Some(family),
            10_000,
        );
    }

    diagnostics.compute_statistics();

    println!("Diagnostic Summary:");
    println!("  Total scans: {}", diagnostics.total_count);
    println!("  Sampled: {}", diagnostics.sample_count);
    println!("\n  Error (ppm):");
    println!(
        "    Before: mean = {:.4}, mae = {:.4}, rms = {:.4}, max = {:.4}",
        diagnostics.mean_error_ppm_before,
        diagnostics.mae_error_ppm_before,
        diagnostics.rms_error_ppm_before,
        diagnostics.max_abs_error_ppm_before,
    );
    println!(
        "    After:  mean = {:.4}, mae = {:.4}, rms = {:.4}, max = {:.4}",
        diagnostics.mean_error_ppm_after,
        diagnostics.mae_error_ppm_after,
        diagnostics.rms_error_ppm_after,
        diagnostics.max_abs_error_ppm_after,
    );
    println!(
        "    Improvement: {:.4} ppm (MAE), {:.4} ppm (RMS)",
        diagnostics.mean_error_improvement_ppm(),
        diagnostics.rms_error_improvement_ppm(),
    );

    println!("\n  By adduct family:");
    for (family, errors) in &diagnostics.error_by_adduct_before {
        println!("    {}: {} scans", family, errors.len());
    }

    diagnostics
}

fn run_visualization_example(diagnostics: &RecalibrationStats) {
    println!("\n=== Example 5: Diagnostic Visualization ===");

    let scatter_plot = render_recalibration_diagnostic_ppm(
        &diagnostics.error_ppm_before,
        &diagnostics.error_ppm_after,
    )
    .unwrap_or_default();
    println!("Generated scatter plot: {} bytes SVG", scatter_plot.len());

    let histogram = render_recalibration_diagnostic_histogram(
        &diagnostics.error_ppm_before,
        &diagnostics.error_ppm_after,
        10,
    )
    .unwrap_or_default();
    println!("Generated histogram: {} bytes SVG", histogram.len());
    println!(
        "MAE improvement: {:.4} ppm",
        diagnostics.mean_error_improvement_ppm()
    );
}

/// The m/z of the single peak a demo fragment list holds.
///
/// `recalibrate_fragments` takes a slice, so the two "unchanged" demos below keep
/// their one peak in a `Vec` and read it back through this. An empty list prints
/// as `NaN`, which is what it is: there is no m/z to report.
fn sole_mz(fragments: &[Peak]) -> f64 {
    fragments.first().map_or(f64::NAN, |peak| peak.mz)
}

fn run_no_correction_example(precursor_ms1: f64, precursor_ms2: f64) {
    println!("\n=== Example 6: No Correction (λ = 0) ===");

    let mut fragments = vec![Peak::new(100.0, 5000.0)];
    let original_mz = sole_mz(&fragments);

    recalibrate_fragments(
        &mut fragments,
        precursor_ms1,
        precursor_ms2,
        CalibrationModel::tof(0.0),
    );

    let after_mz = sole_mz(&fragments);
    println!("With λ = 0.0, fragment m/z should be unchanged:");
    println!("  Before: {original_mz:.6}");
    println!("  After:  {after_mz:.6}");
    println!("  Equal: {}", (original_mz - after_mz).abs() < 1e-10);
}

fn run_disabled_example(precursor_ms1: f64, precursor_ms2: f64) {
    println!("\n=== Example 7: Disabled Recalibration ===");

    let mut fragments = vec![Peak::new(100.0, 5000.0)];
    let original_mz = sole_mz(&fragments);

    recalibrate_fragments(
        &mut fragments,
        precursor_ms1,
        precursor_ms2,
        CalibrationModel::None,
    );

    let after_mz = sole_mz(&fragments);
    println!("With CalibrationModel::None, fragment m/z is unchanged:");
    println!("  Before: {original_mz:.6}");
    println!("  After:  {after_mz:.6}");
    println!("  Equal: {}", (original_mz - after_mz).abs() < 1e-10);
}

fn main() {
    let precursor_ms1 = 500.0000;
    let precursor_ms2 = 500.0120;

    run_tof_examples(precursor_ms1, precursor_ms2);
    run_orbitrap_example();
    let diagnostics = run_diagnostics_example();
    run_visualization_example(&diagnostics);
    run_no_correction_example(precursor_ms1, precursor_ms2);
    run_disabled_example(precursor_ms1, precursor_ms2);

    println!("\n=== All Examples Complete ===");
}
