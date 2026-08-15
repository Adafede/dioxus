// Example: Using Precursor-Driven MS2 Fragment Recalibration
//
// This example demonstrates how to use the recalibration module to:
// 1. Define a calibration model
// 2. Recalibrate fragments
// 3. Track diagnostic metrics
// 4. Generate visualization plots

use mgf_precursor_erro_rs::diagnostics::RecalibrationStats;
use mgf_precursor_erro_rs::plotting::{
    render_recalibration_diagnostic_histogram, render_recalibration_diagnostic_ppm,
    render_recalibration_summary_text,
};
use mgf_precursor_erro_rs::recalibration::{CalibrationModel, Peak, recalibrate_fragments};

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
        CalibrationModel::TOFDa { lambda: 1.0 },
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
        CalibrationModel::TOFDa { lambda: 0.5 },
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
        CalibrationModel::OrbitrapPPM { lambda: 1.0 },
    );

    print_fragments("\nAfter recalibration (λ = 1.0):", &fragments, false);
}

fn run_diagnostics_example() -> RecalibrationStats {
    println!("\n=== Example 4: Diagnostic Metrics Tracking ===");

    let mut diagnostics = RecalibrationStats::new();
    let errors = [
        (10.5, 5.0, 0.1050, 0.0500, "protonated"),
        (12.0, 6.0, 0.1200, 0.0600, "protonated"),
        (-8.5, -4.0, -0.0850, -0.0400, "deprotonated"),
        (9.5, 4.5, 0.0950, 0.0450, "protonated"),
        (11.0, 5.5, 0.1100, 0.0550, "deprotonated"),
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
        "    Before: mean = {:.4}, rms = {:.4}, max = {:.4}",
        diagnostics.mean_error_ppm_before,
        diagnostics.rms_error_ppm_before,
        diagnostics.max_abs_error_ppm_before,
    );
    println!(
        "    After:  mean = {:.4}, rms = {:.4}, max = {:.4}",
        diagnostics.mean_error_ppm_after,
        diagnostics.rms_error_ppm_after,
        diagnostics.max_abs_error_ppm_after,
    );
    println!(
        "    Improvement: {:.4} ppm (mean), {:.4} ppm (rms)",
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

    let errors_before = vec![10.0, 12.0, -8.5, 9.5, 11.0];
    let errors_after = vec![5.0, 6.0, -4.0, 4.5, 5.5];

    let scatter_plot =
        render_recalibration_diagnostic_ppm(&errors_before, &errors_after).unwrap_or_default();
    println!("Generated scatter plot: {} bytes SVG", scatter_plot.len());

    let histogram = render_recalibration_diagnostic_histogram(&errors_before, &errors_after, 10)
        .unwrap_or_default();
    println!("Generated histogram: {} bytes SVG", histogram.len());

    let summary = render_recalibration_summary_text(
        diagnostics.mean_error_ppm_before,
        diagnostics.mean_error_ppm_after,
        diagnostics.rms_error_ppm_before,
        diagnostics.rms_error_ppm_after,
        diagnostics.max_abs_error_ppm_before,
        diagnostics.max_abs_error_ppm_after,
    )
    .unwrap_or_default();
    println!("Generated summary: {} bytes HTML", summary.len());
}

fn run_no_correction_example(precursor_ms1: f64, precursor_ms2: f64) {
    println!("\n=== Example 6: No Correction (λ = 0) ===");

    let mut fragments = vec![Peak::new(100.0, 5000.0)];
    let original_mz = fragments[0].mz;

    recalibrate_fragments(
        &mut fragments,
        precursor_ms1,
        precursor_ms2,
        CalibrationModel::TOFDa { lambda: 0.0 },
    );

    println!("With λ = 0.0, fragment m/z should be unchanged:");
    println!("  Before: {original_mz:.6}");
    println!("  After:  {:.6}", fragments[0].mz);
    println!("  Equal: {}", (original_mz - fragments[0].mz).abs() < 1e-10);
}

fn run_disabled_example(precursor_ms1: f64, precursor_ms2: f64) {
    println!("\n=== Example 7: Disabled Recalibration ===");

    let mut fragments = vec![Peak::new(100.0, 5000.0)];
    let original_mz = fragments[0].mz;

    recalibrate_fragments(
        &mut fragments,
        precursor_ms1,
        precursor_ms2,
        CalibrationModel::None,
    );

    println!("With CalibrationModel::None, fragment m/z is unchanged:");
    println!("  Before: {original_mz:.6}");
    println!("  After:  {:.6}", fragments[0].mz);
    println!("  Equal: {}", (original_mz - fragments[0].mz).abs() < 1e-10);
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
