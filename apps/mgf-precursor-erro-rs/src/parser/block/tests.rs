// The `tests` tests, extracted from `block.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `block` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;

#[test]
fn keeps_full_precision_when_computing_error() {
    let state = BlockParseState {
        observed_precursor_raw: Some("100.1234".to_string()),
        observed_precursor: Some(100.1234),
        reference_mass: Some(100.123_456_789),
        reference_mass_source: Some("EXACTMASS".to_string()),
        charge: Some("1".to_string()),
        ..Default::default()
    };

    let mut smiles_cache = HashMap::new();
    let mut formula_cache = HashMap::new();
    let mut logged_failures = HashSet::new();
    let mut plot_sample = None;

    let metrics = process_block_state(
        &state,
        &mut smiles_cache,
        &mut formula_cache,
        &mut logged_failures,
        &mut plot_sample,
    )
    .expect("metrics should be produced");

    let expected_error = 100.1234_f64 - 100.123_456_789_f64;
    assert!((metrics.sample_abs_error_da - expected_error.abs()).abs() < 1e-12);
}
