// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

/// Represents different calibration models for MS2 fragment recalibration.
///
/// This module provides a generic interface for applying scan-wide calibration corrections
/// to fragment peaks based on the discrepancy between the precursor measured in MS1 and
/// the precursor reported in the MS2 scan.
///
/// The precursor discrepancy is treated as an estimate of a latent scan-wide calibration error,
/// and a shrinkage parameter λ (lambda) controls what fraction of the error is applied.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum CalibrationModel {
    /// No recalibration is applied.
    #[default]
    None,

    /// Orbitrap-style recalibration using parts-per-million (ppm).
    ///
    /// The error is computed as:
    /// ```ignore
    /// delta_ppm = 1e6 * (precursor_ms2 - precursor_ms1) / precursor_ms1
    /// ```
    ///
    /// Each fragment is corrected as:
    /// ```ignore
    /// fragment_corrected = fragment * (1 - λ * delta_ppm / 1e6)
    /// ```
    ///
    /// # Parameters
    /// - `lambda`: Shrinkage parameter in [0, 1].
    ///   - 0 = no correction
    ///   - 1 = full precursor transfer
    ///   - intermediate = shrinkage estimator
    OrbitrapPPM { lambda: f64 },

    /// Time-of-flight-style recalibration using absolute mass difference in Da.
    ///
    /// The error is computed as:
    /// ```ignore
    /// delta_da = precursor_ms2 - precursor_ms1
    /// ```
    ///
    /// Each fragment is corrected as:
    /// ```ignore
    /// fragment_corrected = fragment - λ * delta_da
    /// ```
    ///
    /// # Parameters
    /// - `lambda`: Shrinkage parameter in [0, 1].
    ///   - 0 = no correction
    ///   - 1 = full precursor transfer
    ///   - intermediate = shrinkage estimator
    TOFDa { lambda: f64 },
}

impl CalibrationModel {
    /// Returns whether this model applies any correction.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns the lambda parameter if applicable, otherwise None.
    #[must_use]
    pub const fn lambda(&self) -> Option<f64> {
        match self {
            Self::None => None,
            Self::OrbitrapPPM { lambda } | Self::TOFDa { lambda } => Some(*lambda),
        }
    }
}

/// A simple struct representing a mass spectrometry peak (fragment).
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Peak {
    /// The m/z value of the peak.
    pub mz: f64,
    /// The intensity of the peak.
    pub intensity: f64,
}

impl Peak {
    /// Creates a new peak with the given m/z and intensity.
    #[must_use]
    pub const fn new(mz: f64, intensity: f64) -> Self {
        Self { mz, intensity }
    }
}
