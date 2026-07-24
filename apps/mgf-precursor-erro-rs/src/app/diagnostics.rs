use dioxus::prelude::Signal;
#[cfg(target_arch = "wasm32")]
use dioxus::prelude::WritableExt;

use crate::diagnostics::RecalibrationStats;
use crate::metrics::PrecursorStats;
use crate::recalibration::CalibrationModel;

#[cfg(target_arch = "wasm32")]
pub fn update_recalibration_diagnostics(
    metrics: &PrecursorStats,
    model: CalibrationModel,
    diagnostics_signal: &mut Signal<Option<RecalibrationStats>>,
) {
    if matches!(model, CalibrationModel::None) {
        diagnostics_signal.set(None);
        return;
    }

    let mut diag = RecalibrationStats::new();
    let lambda = match model {
        CalibrationModel::TOFDa { lambda } => lambda,
        CalibrationModel::OrbitrapPPM { lambda } => lambda,
        _ => 0.0,
    };

    for point in &metrics.plot_points {
        let Some(theoretical_mass) = point.expected_mass else {
            continue;
        };

        let precursor_ms2 = point.ms2_precursor_peak.unwrap_or(point.pepmass_header);
        let precursor_ms1 = point.pepmass_header;
        let error_da_ms1 = precursor_ms1 - theoretical_mass;
        let error_ppm_ms1 = if theoretical_mass > 0.0 {
            error_da_ms1 * 1e6 / theoretical_mass
        } else {
            0.0
        };

        let error_da_before = precursor_ms2 - theoretical_mass;
        let error_ppm_before = if theoretical_mass > 0.0 {
            error_da_before * 1e6 / theoretical_mass
        } else {
            0.0
        };

        let delta_ms2_ms1_da = precursor_ms2 - precursor_ms1;
        let delta_ppm_ms2_ms1 = if precursor_ms1 > 0.0 {
            delta_ms2_ms1_da * 1e6 / precursor_ms1
        } else {
            0.0
        };

        let precursor_ms2_after = match model {
            CalibrationModel::TOFDa { .. } => precursor_ms2 - lambda * delta_ms2_ms1_da,
            CalibrationModel::OrbitrapPPM { .. } => {
                precursor_ms2 * (1.0 - lambda * delta_ppm_ms2_ms1 / 1e6)
            }
            _ => precursor_ms2,
        };

        let error_da_after = precursor_ms2_after - theoretical_mass;
        let error_ppm_after = if theoretical_mass > 0.0 {
            error_da_after * 1e6 / theoretical_mass
        } else {
            0.0
        };

        let adduct_str = match point.adduct_family {
            crate::metrics::AdductFamily::Protonated => Some("[M+H]+"),
            crate::metrics::AdductFamily::Deprotonated => Some("[M-H]-"),
            crate::metrics::AdductFamily::AlkaliAmmonium => Some("[M+NH4]+"),
            crate::metrics::AdductFamily::MetalComplex => Some("[M+Metal]"),
            crate::metrics::AdductFamily::Halide => Some("[M-Hal]"),
            crate::metrics::AdductFamily::Other => None,
        };

        diag.push_measurement(crate::diagnostics::RecalibrationMeasurement {
            error_ppm_ms1,
            delta_ppm_ms2_ms1,
            error_ppm_before,
            error_ppm_after,
            error_da_ms1,
            delta_da_ms2_ms1: delta_ms2_ms1_da,
            error_da_before,
            error_da_after,
            precursor_ms1,
            precursor_ms2_before: precursor_ms2,
            precursor_ms2_after,
            adduct_family: adduct_str,
            max_samples: 5000,
        });
    }

    diag.compute_statistics();
    diagnostics_signal.set(Some(diag));
}

#[cfg(not(target_arch = "wasm32"))]
pub const fn update_recalibration_diagnostics(
    _metrics: &PrecursorStats,
    _model: CalibrationModel,
    _diagnostics_signal: &mut Signal<Option<RecalibrationStats>>,
) {
}
