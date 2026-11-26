// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Precursor error metric update helpers for [`PrecursorStats`].

use super::stats::{PrecursorStats, push_reservoir};
use super::{ErrorMeasurement, HighErrorSmilesDetail, MAX_ECDF_POINTS, MAX_PLOT_POINTS, PlotPoint};
use crate::numeric::usize_to_f64;

const HIGH_ERROR_THRESHOLD_DA: f64 = 0.01;

impl PrecursorStats {
    /// Records a single spectrum error observation into all running metrics and samplers.
    pub(crate) fn record_error(&mut self, measurement: ErrorMeasurement<'_>) {
        if !measurement.abs_error_da.is_finite()
            || !measurement.abs_ppm.is_finite()
            || !measurement.pepmass_header.is_finite()
        {
            return;
        }
        self.total_spectra += 1;
        self.spectra += 1;
        self.spectra_with_reference_mass += 1;
        let total_spectra = usize_to_f64(self.spectra);

        self.update_extrema_and_moments(
            measurement.pepmass_header,
            measurement.abs_error_da,
            measurement.abs_ppm,
            total_spectra,
        );
        push_reservoir(
            measurement.abs_error_da,
            &mut self.absolute_error_da_values,
            &mut self.absolute_error_da_sample_seen,
            MAX_ECDF_POINTS,
        );
        push_reservoir(
            measurement.abs_ppm,
            &mut self.absolute_error_ppm_values,
            &mut self.absolute_error_ppm_sample_seen,
            MAX_ECDF_POINTS,
        );
        self.observed_precursor_median_tracker
            .push(measurement.pepmass_header);
        self.abs_error_da_median_tracker
            .push(measurement.abs_error_da);
        self.abs_error_ppm_median_tracker.push(measurement.abs_ppm);
        self.observed_precursor_median = self.observed_precursor_median_tracker.median();
        self.abs_error_da_median = self.abs_error_da_median_tracker.median();
        self.abs_error_ppm_median = self.abs_error_ppm_median_tracker.median();

        self.update_error_buckets(measurement.abs_error_da * 1000.0, measurement.abs_ppm);
        self.update_high_error_smiles(&measurement);

        push_reservoir(
            PlotPoint {
                adduct_family: measurement.adduct_family,
                pepmass_header: measurement.pepmass_header,
                ms2_precursor_peak: measurement.ms2_precursor_peak,
                signed_error_da: measurement.signed_error_da,
                signed_error_ppm: measurement.ppm_error,
                expected_mass: measurement.expected_mass,
            },
            &mut self.plot_points,
            &mut self.plot_point_stream_seen,
            MAX_PLOT_POINTS,
        );
    }

    fn update_extrema_and_moments(
        &mut self,
        pepmass_header: f64,
        abs_error_da: f64,
        abs_ppm: f64,
        total_spectra: f64,
    ) {
        if self.spectra == 1 {
            self.observed_precursor_min = pepmass_header;
            self.observed_precursor_max = pepmass_header;
            self.abs_error_da_min = abs_error_da;
            self.abs_error_da_max = abs_error_da;
            self.abs_error_ppm_min = abs_ppm;
            self.abs_error_ppm_max = abs_ppm;
        } else {
            self.observed_precursor_min = self.observed_precursor_min.min(pepmass_header);
            self.observed_precursor_max = self.observed_precursor_max.max(pepmass_header);
            self.abs_error_da_min = self.abs_error_da_min.min(abs_error_da);
            self.abs_error_da_max = self.abs_error_da_max.max(abs_error_da);
            self.abs_error_ppm_min = self.abs_error_ppm_min.min(abs_ppm);
            self.abs_error_ppm_max = self.abs_error_ppm_max.max(abs_ppm);
        }

        self.observed_precursor_mean +=
            (pepmass_header - self.observed_precursor_mean) / total_spectra;
        self.abs_error_da_mean += (abs_error_da - self.abs_error_da_mean) / total_spectra;
        let da_sq = self.abs_error_da_rms * self.abs_error_da_rms;
        self.abs_error_da_rms =
            (da_sq + (abs_error_da * abs_error_da - da_sq) / total_spectra).sqrt();

        self.abs_error_ppm_mean += (abs_ppm - self.abs_error_ppm_mean) / total_spectra;
        let ppm_sq = self.abs_error_ppm_rms * self.abs_error_ppm_rms;
        self.abs_error_ppm_rms = (ppm_sq + (abs_ppm * abs_ppm - ppm_sq) / total_spectra).sqrt();
    }

    fn update_error_buckets(&mut self, abs_error_mda: f64, abs_ppm: f64) {
        match abs_error_mda {
            v if v <= 0.1 => self.within_0_1_mda = self.within_0_1_mda.saturating_add(1),
            v if v <= 0.5 => self.within_0_5_mda = self.within_0_5_mda.saturating_add(1),
            v if v <= 1.0 => self.within_1_mda = self.within_1_mda.saturating_add(1),
            v if v <= 5.0 => self.within_5_mda = self.within_5_mda.saturating_add(1),
            _ => self.above_5_mda = self.above_5_mda.saturating_add(1),
        }

        match abs_ppm {
            v if v <= 0.5 => self.within_0_5_ppm = self.within_0_5_ppm.saturating_add(1),
            v if v <= 1.0 => self.within_1_ppm = self.within_1_ppm.saturating_add(1),
            v if v <= 5.0 => self.within_5_ppm = self.within_5_ppm.saturating_add(1),
            v if v <= 10.0 => self.within_10_ppm = self.within_10_ppm.saturating_add(1),
            _ => self.above_10_ppm = self.above_10_ppm.saturating_add(1),
        }
    }

    fn update_high_error_smiles(&mut self, update: &ErrorMeasurement<'_>) {
        let Some(smiles) = update.smiles.map(str::trim).filter(|s| !s.is_empty()) else {
            return;
        };
        if update.abs_error_da <= HIGH_ERROR_THRESHOLD_DA {
            return;
        }
        if let Some(entry) = self.high_error_smiles.get_mut(smiles) {
            entry.count = entry.count.saturating_add(1);
            if entry.calculated_mass.is_none() && update.calculated_mass.is_some() {
                entry.calculated_mass = update.calculated_mass;
            }
            if entry.expected_mass.is_none() && update.expected_mass.is_some() {
                entry.expected_mass = update.expected_mass;
            }
            if entry.formula.is_none() {
                entry.formula = update.formula.map(str::to_string);
            }
            if entry.observed_precursor_mz.is_none() {
                entry.observed_precursor_mz = Some(update.pepmass_header);
            }
            if entry
                .max_abs_error_da
                .is_none_or(|existing| update.abs_error_da > existing)
            {
                entry.max_abs_error_da = Some(update.abs_error_da);
                entry.max_abs_error_ppm = Some(update.abs_ppm);
                if update.calculated_mass.is_some() {
                    entry.calculated_mass = update.calculated_mass;
                }
                if update.expected_mass.is_some() {
                    entry.expected_mass = update.expected_mass;
                }
                if update.formula.is_some() {
                    entry.formula = update.formula.map(str::to_string);
                }
                entry.observed_precursor_mz = Some(update.pepmass_header);
            }
            return;
        }
        self.high_error_smiles.insert(
            smiles.to_owned(),
            HighErrorSmilesDetail {
                count: 1,
                calculated_mass: update.calculated_mass,
                expected_mass: update.expected_mass,
                formula: update.formula.map(str::to_string),
                max_abs_error_da: Some(update.abs_error_da),
                max_abs_error_ppm: Some(update.abs_ppm),
                observed_precursor_mz: Some(update.pepmass_header),
            },
        );
    }
}
