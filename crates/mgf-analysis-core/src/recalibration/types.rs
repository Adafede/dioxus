// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Value types for MS2 fragment recalibration (`ShrinkageLambda`, `CalibrationModel`, `Peak`).

use crate::error::MgfAnalysisError;

/// Validated shrinkage weight `lambda` in `[0.0, 1.0]` controlling precursor correction strength.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShrinkageLambda(f64);

impl ShrinkageLambda {
    /// Zero correction (`lambda = 0.0`).
    pub const ZERO: Self = Self(0.0);
    /// Default half shrinkage (`lambda = 0.5`).
    pub const HALF: Self = Self(0.5);
    /// Full precursor error transfer (`lambda = 1.0`).
    pub const FULL: Self = Self(1.0);

    /// Validates that `value` is finite and lies in `[0.0, 1.0]`.
    ///
    /// # Errors
    ///
    /// Returns [`MgfAnalysisError::InvalidShrinkageLambda`] when `value` is non-finite or out of bounds.
    pub fn new(value: f64) -> Result<Self, MgfAnalysisError> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(MgfAnalysisError::InvalidShrinkageLambda(value))
        }
    }

    /// Clamps finite values to `[0.0, 1.0]` and maps non-finite inputs to `0.0`.
    #[must_use]
    pub const fn clamped(value: f64) -> Self {
        if !value.is_finite() || value < 0.0 {
            Self(0.0)
        } else if value > 1.0 {
            Self(1.0)
        } else {
            Self(value)
        }
    }

    /// Underlying scalar `lambda` in `[0.0, 1.0]`.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl Default for ShrinkageLambda {
    fn default() -> Self {
        Self::HALF
    }
}

/// Instrument error model paired with its shrinkage parameter [`ShrinkageLambda`].
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum CalibrationModel {
    /// No recalibration is applied.
    #[default]
    None,
    /// Orbitrap-style multiplicative ppm recalibration.
    OrbitrapPPM {
        /// Shrinkage factor `lambda` in `[0.0, 1.0]`.
        lambda: ShrinkageLambda,
    },
    /// Time-of-flight additive Da recalibration.
    TOFDa {
        /// Shrinkage factor `lambda` in `[0.0, 1.0]`.
        lambda: ShrinkageLambda,
    },
}

impl CalibrationModel {
    /// Constructs an [`OrbitrapPPM`](Self::OrbitrapPPM) model with clamped `lambda`.
    #[must_use]
    pub const fn orbitrap(lambda: f64) -> Self {
        Self::OrbitrapPPM {
            lambda: ShrinkageLambda::clamped(lambda),
        }
    }

    /// Constructs a [`TOFDa`](Self::TOFDa) model with clamped `lambda`.
    #[must_use]
    pub const fn tof(lambda: f64) -> Self {
        Self::TOFDa {
            lambda: ShrinkageLambda::clamped(lambda),
        }
    }

    /// Returns a copy of `self` with an updated `lambda` (no-op for [`None`](Self::None)).
    #[must_use]
    pub const fn with_lambda(self, lambda: ShrinkageLambda) -> Self {
        match self {
            Self::None => Self::None,
            Self::OrbitrapPPM { .. } => Self::OrbitrapPPM { lambda },
            Self::TOFDa { .. } => Self::TOFDa { lambda },
        }
    }

    /// Returns `true` when recalibration is enabled.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns the typed [`ShrinkageLambda`] when active.
    #[must_use]
    pub const fn shrinkage(&self) -> Option<ShrinkageLambda> {
        match *self {
            Self::None => None,
            Self::OrbitrapPPM { lambda } | Self::TOFDa { lambda } => Some(lambda),
        }
    }

    /// Returns the scalar `lambda` when active.
    #[must_use]
    pub const fn lambda(&self) -> Option<f64> {
        match *self {
            Self::None => None,
            Self::OrbitrapPPM { lambda } | Self::TOFDa { lambda } => Some(lambda.get()),
        }
    }
}

/// One MS2 fragment ion peak (`m/z` and `intensity`).
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Peak {
    /// Peak `m/z` value.
    pub mz: f64,
    /// Peak intensity.
    pub intensity: f64,
}

impl Peak {
    /// Creates a new fragment peak.
    #[must_use]
    pub const fn new(mz: f64, intensity: f64) -> Self {
        Self { mz, intensity }
    }
}
