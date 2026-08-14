use super::{HighErrorSmilesDetail, PrecursorStats, usize_to_f64};

#[must_use]
pub fn merge_precursor_stats(mut current: PrecursorStats, next: &PrecursorStats) -> PrecursorStats {
    let current_spectra = usize_to_f64(current.spectra);
    let next_spectra = usize_to_f64(next.spectra);
    let total_spectra = current_spectra + next_spectra;

    merge_precursor_counts(&mut current, next);
    merge_precursor_source(&mut current, next);
    merge_precursor_scalar_stats(
        &mut current,
        next,
        current_spectra,
        next_spectra,
        total_spectra,
    );
    merge_precursor_histograms(&mut current, next);
    merge_precursor_samples(&mut current, next);
    merge_precursor_plot_points(&mut current, next);
    merge_precursor_maps(&mut current, next);
    current
}

const fn merge_precursor_counts(current: &mut PrecursorStats, next: &PrecursorStats) {
    current.spectra += next.spectra;
    current.total_spectra += next.total_spectra;
    current.skipped_spectra += next.skipped_spectra;
    current.spectra_with_reference_mass += next.spectra_with_reference_mass;
    current.unparsed_smiles += next.unparsed_smiles;
}

fn merge_precursor_source(current: &mut PrecursorStats, next: &PrecursorStats) {
    if current.reference_mass_source == "none" {
        current
            .reference_mass_source
            .clone_from(&next.reference_mass_source);
    } else if current.reference_mass_source != next.reference_mass_source
        && !next.reference_mass_source.is_empty()
    {
        current.reference_mass_source = "mixed".to_string();
    }
}

fn merge_precursor_scalar_stats(
    current: &mut PrecursorStats,
    next: &PrecursorStats,
    current_spectra: f64,
    next_spectra: f64,
    total_spectra: f64,
) {
    current.observed_precursor_min = current
        .observed_precursor_min
        .min(next.observed_precursor_min);
    current.observed_precursor_max = current
        .observed_precursor_max
        .max(next.observed_precursor_max);
    current.observed_precursor_mean = f64::mul_add(
        current.observed_precursor_mean,
        current_spectra,
        next.observed_precursor_mean * next_spectra,
    ) / total_spectra;
    current
        .observed_precursor_median_tracker
        .merge(next.observed_precursor_median_tracker.clone());
    current.observed_precursor_median = current.observed_precursor_median_tracker.median();

    current.abs_error_da_min = current.abs_error_da_min.min(next.abs_error_da_min);
    current.abs_error_da_max = current.abs_error_da_max.max(next.abs_error_da_max);
    current.abs_error_da_mean = f64::mul_add(
        current.abs_error_da_mean,
        current_spectra,
        next.abs_error_da_mean * next_spectra,
    ) / total_spectra;
    current
        .abs_error_da_median_tracker
        .merge(next.abs_error_da_median_tracker.clone());
    current.abs_error_da_median = current.abs_error_da_median_tracker.median();
    let current_da_rms_sq = current.abs_error_da_rms * current.abs_error_da_rms;
    let next_da_rms_sq = next.abs_error_da_rms * next.abs_error_da_rms;
    current.abs_error_da_rms = f64::mul_add(
        current_da_rms_sq,
        current_spectra,
        next_da_rms_sq * next_spectra,
    ) / total_spectra;
    current.abs_error_da_rms = current.abs_error_da_rms.sqrt();

    current.abs_error_ppm_min = current.abs_error_ppm_min.min(next.abs_error_ppm_min);
    current.abs_error_ppm_max = current.abs_error_ppm_max.max(next.abs_error_ppm_max);
    current.abs_error_ppm_mean = f64::mul_add(
        current.abs_error_ppm_mean,
        current_spectra,
        next.abs_error_ppm_mean * next_spectra,
    ) / total_spectra;
    current
        .abs_error_ppm_median_tracker
        .merge(next.abs_error_ppm_median_tracker.clone());
    current.abs_error_ppm_median = current.abs_error_ppm_median_tracker.median();
    let current_ppm_rms_sq = current.abs_error_ppm_rms * current.abs_error_ppm_rms;
    let next_ppm_rms_sq = next.abs_error_ppm_rms * next.abs_error_ppm_rms;
    current.abs_error_ppm_rms = f64::mul_add(
        current_ppm_rms_sq,
        current_spectra,
        next_ppm_rms_sq * next_spectra,
    ) / total_spectra;
    current.abs_error_ppm_rms = current.abs_error_ppm_rms.sqrt();

    current.signed_error_da_mean = f64::mul_add(
        current.signed_error_da_mean,
        current_spectra,
        next.signed_error_da_mean * next_spectra,
    ) / total_spectra;
    current
        .signed_error_da_median_tracker
        .merge(next.signed_error_da_median_tracker.clone());
    current.signed_error_da_median = current.signed_error_da_median_tracker.median();
    current.signed_error_ppm_mean = f64::mul_add(
        current.signed_error_ppm_mean,
        current_spectra,
        next.signed_error_ppm_mean * next_spectra,
    ) / total_spectra;
    current
        .signed_error_ppm_median_tracker
        .merge(next.signed_error_ppm_median_tracker.clone());
    current.signed_error_ppm_median = current.signed_error_ppm_median_tracker.median();
}

fn merge_precursor_histograms(current: &mut PrecursorStats, next: &PrecursorStats) {
    current.within_0_1_da += next.within_0_1_da;
    current.within_0_5_da += next.within_0_5_da;
    current.within_1_da += next.within_1_da;
    current.within_5_da += next.within_5_da;
    current.within_0_5_ppm += next.within_0_5_ppm;
    current.within_1_ppm += next.within_1_ppm;
    current.within_5_ppm += next.within_5_ppm;
    current.above_5_da += next.above_5_da;
    current.within_10_ppm += next.within_10_ppm;
    current.above_10_ppm += next.above_10_ppm;

    for (current_count, count) in current
        .da_error_histogram
        .bins
        .iter_mut()
        .zip(next.da_error_histogram.bins.iter())
    {
        *current_count += count;
    }
    for (current_count, count) in current
        .ppm_error_histogram
        .bins
        .iter_mut()
        .zip(next.ppm_error_histogram.bins.iter())
    {
        *current_count += count;
    }
}

fn merge_precursor_samples(current: &mut PrecursorStats, next: &PrecursorStats) {
    for value in next.absolute_error_da_values.iter().copied() {
        PrecursorStats::push_sampled_value(
            value,
            &mut current.absolute_error_da_values,
            &mut current.absolute_error_da_sample_seen,
        );
    }

    for value in next.absolute_error_ppm_values.iter().copied() {
        PrecursorStats::push_sampled_value(
            value,
            &mut current.absolute_error_ppm_values,
            &mut current.absolute_error_ppm_sample_seen,
        );
    }
}

fn merge_precursor_plot_points(current: &mut PrecursorStats, next: &PrecursorStats) {
    for point in next.plot_points.iter().copied() {
        current.push_plot_point(point);
    }
}

fn merge_precursor_maps(current: &mut PrecursorStats, next: &PrecursorStats) {
    for (smiles, detail) in &next.unparsed_smiles_warnings {
        current
            .unparsed_smiles_warnings
            .entry(smiles.clone())
            .and_modify(|existing| {
                existing.count = existing.count.saturating_add(detail.count);
                if existing.formula.is_none() {
                    existing.formula.clone_from(&detail.formula);
                }
            })
            .or_insert_with(|| (*detail).clone());
    }

    for (adduct, count) in &next.unrecognized_adducts {
        current
            .unrecognized_adducts
            .entry(adduct.clone())
            .and_modify(|existing| *existing += count)
            .or_insert(*count);
    }

    for (smiles, detail) in &next.high_error_smiles {
        current
            .high_error_smiles
            .entry(smiles.clone())
            .and_modify(|existing| merge_high_error_detail(existing, detail))
            .or_insert_with(|| (*detail).clone());
    }
}

fn merge_high_error_detail(existing: &mut HighErrorSmilesDetail, detail: &HighErrorSmilesDetail) {
    existing.count = existing.count.saturating_add(detail.count);
    if existing.calculated_mass.is_none() && detail.calculated_mass.is_some() {
        existing.calculated_mass = detail.calculated_mass;
    }
    if existing.expected_mass.is_none() && detail.expected_mass.is_some() {
        existing.expected_mass = detail.expected_mass;
    }
    if existing.formula.is_none() {
        existing.formula.clone_from(&detail.formula);
    }
    if existing.observed_precursor_mz.is_none() && detail.observed_precursor_mz.is_some() {
        existing.observed_precursor_mz = detail.observed_precursor_mz;
    }
    if let Some(detail_error_da) = detail.max_abs_error_da
        && existing
            .max_abs_error_da
            .is_none_or(|existing_error| detail_error_da > existing_error)
    {
        existing.max_abs_error_da = Some(detail_error_da);
        existing.max_abs_error_ppm = detail.max_abs_error_ppm;
        if detail.calculated_mass.is_some() {
            existing.calculated_mass = detail.calculated_mass;
        }
        if detail.expected_mass.is_some() {
            existing.expected_mass = detail.expected_mass;
        }
        if detail.observed_precursor_mz.is_some() {
            existing.observed_precursor_mz = detail.observed_precursor_mz;
        }
    }
}
