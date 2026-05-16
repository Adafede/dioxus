// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Form input validation framework.
//!
//! Centralizes validation logic for search form inputs to enable:
//! * Consistent validation rules across components
//! * Reusable validators for common patterns (ranges, numbers, strings)
//! * Type-safe validation results
//! * Clear error messages
//!
//! ## Pattern: Validator Functions
//!
//! Each validator is a pure function that takes raw input and returns a `Result`.
//! Validators can be composed to build complex validation pipelines.

use crate::features::explore::types::ValidationFault;
use crate::models::SearchCriteria;

/// Validation result for a form field.
#[allow(dead_code)] // Public API for future form validation wiring
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Strongly-typed validation code used for fault mapping and UI error routing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationCode {
    TaxonTooLong,
    StructureTooLong,
    MassOutOfRange,
    MinGreaterThanMax,
    YearOutOfRange,
    YearRangeInvalid,
    ElementCountHighWarning,
    MassRangeInvalid,
}

/// Strongly-typed field identifier for validation targeting in UI forms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationField {
    Taxon,
    Smiles,
    Mass,
    Year,
    Formula,
}

/// A validation error with localization context.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)] // Members accessed via public API in future phases
pub struct ValidationError {
    /// Field name for error targeting.
    pub field: ValidationField,
    /// Typed validation code (stable contract for domain fault mapping).
    pub code: ValidationCode,
}

impl ValidationError {
    #[allow(dead_code)]
    pub fn new(field: ValidationField, code: ValidationCode) -> Self {
        Self { field, code }
    }
}

fn validation_fault_from_error(error: &ValidationError) -> ValidationFault {
    match error.code {
        ValidationCode::TaxonTooLong => ValidationFault::TaxonTooLong,
        ValidationCode::StructureTooLong => ValidationFault::StructureTooLong,
        ValidationCode::MassOutOfRange => ValidationFault::MassOutOfRange,
        ValidationCode::MinGreaterThanMax | ValidationCode::MassRangeInvalid => {
            ValidationFault::MassRangeInvalid
        }
        ValidationCode::YearOutOfRange => ValidationFault::YearOutOfRange,
        ValidationCode::YearRangeInvalid => ValidationFault::YearRangeInvalid,
        ValidationCode::ElementCountHighWarning => ValidationFault::ElementCountTooHigh,
    }
}

// ── Validators for common patterns ───────────────────────────────────────────

/// Validate that a taxon string is not empty and within reasonable bounds.
#[allow(dead_code)]
pub fn validate_taxon(input: &str) -> ValidationResult<()> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Ok(()); // Optional field
    }

    if trimmed.len() > 500 {
        return Err(ValidationError::new(
            ValidationField::Taxon,
            ValidationCode::TaxonTooLong,
        ));
    }

    Ok(())
}

/// Validate that a SMILES/Molfile string is not too long.
#[allow(dead_code)]
pub fn validate_smiles(input: &str) -> ValidationResult<()> {
    if input.is_empty() {
        return Ok(()); // Optional field
    }

    if input.len() > 10_000 {
        return Err(ValidationError::new(
            ValidationField::Smiles,
            ValidationCode::StructureTooLong,
        ));
    }

    Ok(())
}

/// Validate that a mass value is within valid range.
#[allow(dead_code)]
pub fn validate_mass(value: f64, min_max: (f64, f64)) -> ValidationResult<()> {
    let (min, max) = min_max;

    if !(0.0..=10_000.0).contains(&value) {
        return Err(ValidationError::new(
            ValidationField::Mass,
            ValidationCode::MassOutOfRange,
        ));
    }

    if min > max {
        return Err(ValidationError::new(
            ValidationField::Mass,
            ValidationCode::MinGreaterThanMax,
        ));
    }

    Ok(())
}

/// Validate that a year value is within reasonable range.
#[allow(dead_code)]
pub fn validate_year(value: u16) -> ValidationResult<()> {
    const MIN_YEAR: u16 = 1600;
    let current_year = crate::models::current_year();

    if !(MIN_YEAR..=current_year).contains(&value) {
        return Err(ValidationError::new(
            ValidationField::Year,
            ValidationCode::YearOutOfRange,
        ));
    }

    Ok(())
}

/// Validate that a year range is sensible.
#[allow(dead_code)]
pub fn validate_year_range(min: u16, max: u16) -> ValidationResult<()> {
    validate_year(min)?;
    validate_year(max)?;

    if min > max {
        return Err(ValidationError::new(
            ValidationField::Year,
            ValidationCode::YearRangeInvalid,
        ));
    }

    Ok(())
}

/// Validate that an element count is within valid range.
#[allow(dead_code)]
pub fn validate_element_count(value: u16) -> ValidationResult<()> {
    if value > 1000 {
        return Err(ValidationError::new(
            ValidationField::Formula,
            ValidationCode::ElementCountHighWarning,
        ));
    }

    Ok(())
}

/// Validate entire search criteria at once.
///
/// Runs all validators and collects results. Returns `Ok(())` if all pass,
/// or `Err(Vec<ValidationError>)` if any fail.
#[allow(dead_code)]
pub fn validate_criteria(criteria: &SearchCriteria) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    if let Err(e) = validate_taxon(&criteria.taxon) {
        errors.push(e);
    }

    if let Err(e) = validate_smiles(&criteria.smiles) {
        errors.push(e);
    }

    if let Err(e) = validate_mass(criteria.mass_min, (criteria.mass_min, criteria.mass_max)) {
        errors.push(e);
    }

    if let Err(e) = validate_year_range(criteria.year_min, criteria.year_max) {
        errors.push(e);
    }

    if let Err(e) = validate_element_count(criteria.c_min + criteria.h_min) {
        errors.push(e);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate mass range. Helper for test code and validators.
#[allow(dead_code)]
pub fn validate_mass_range(min: f64, max: f64) -> ValidationResult<()> {
    validate_mass(min, (min, max))?;
    validate_mass(max, (min, max))?;

    if min > max {
        return Err(ValidationError::new(
            ValidationField::Mass,
            ValidationCode::MassRangeInvalid,
        ));
    }

    Ok(())
}

/// Validate criteria at the orchestration boundary.
///
/// This validator returns domain-native `ValidationFault` so `start_search`
/// can fail fast without translating from UI-oriented validation error keys.
pub fn validate_dispatch_criteria(criteria: &SearchCriteria) -> Result<(), ValidationFault> {
    if criteria.taxon.trim().is_empty()
        && criteria.smiles.trim().is_empty()
        && !criteria.formula_enabled
    {
        return Err(ValidationFault::EmptyInput);
    }
    if let Err(errors) = validate_criteria(criteria)
        && let Some(first) = errors.first()
    {
        return Err(validation_fault_from_error(first));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_taxon_accepts_empty_string() {
        assert!(validate_taxon("").is_ok());
    }

    #[test]
    fn validate_taxon_accepts_valid_input() {
        assert!(validate_taxon("Rosa").is_ok());
    }

    #[test]
    fn validate_taxon_rejects_extremely_long_input() {
        let long = "x".repeat(501);
        assert!(validate_taxon(&long).is_err());
    }

    #[test]
    fn validate_smiles_accepts_empty_string() {
        assert!(validate_smiles("").is_ok());
    }

    #[test]
    fn validate_smiles_accepts_valid_smiles() {
        assert!(validate_smiles("CC(C)Cc1ccc(cc1)C(C)C(=O)O").is_ok());
    }

    #[test]
    fn validate_smiles_rejects_extremely_long_input() {
        let long = "C".repeat(10_001);
        assert!(validate_smiles(&long).is_err());
    }

    #[test]
    fn validate_mass_range_rejects_inverted_range() {
        assert!(validate_mass_range(200.0, 100.0).is_err());
    }

    #[test]
    fn validate_mass_range_accepts_valid_range() {
        assert!(validate_mass_range(100.0, 200.0).is_ok());
    }

    #[test]
    fn validate_year_range_rejects_inverted_range() {
        let result = validate_year_range(2025, 2020);
        assert!(result.is_err());
    }

    #[test]
    fn validate_year_range_accepts_valid_range() {
        let result = validate_year_range(2000, 2025);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_element_count_accepts_reasonable_counts() {
        assert!(validate_element_count(100).is_ok());
    }

    #[test]
    fn validate_element_count_rejects_unreasonably_high_counts() {
        assert!(validate_element_count(1001).is_err());
    }

    #[test]
    fn validate_dispatch_criteria_rejects_empty_primary_filters() {
        let criteria = SearchCriteria {
            taxon: "   ".into(),
            smiles: "".into(),
            formula_enabled: false,
            ..SearchCriteria::default()
        };
        assert_eq!(
            validate_dispatch_criteria(&criteria),
            Err(ValidationFault::EmptyInput)
        );
    }

    #[test]
    fn validate_dispatch_criteria_accepts_formula_only_search() {
        let criteria = SearchCriteria {
            taxon: "".into(),
            smiles: "".into(),
            formula_enabled: true,
            ..SearchCriteria::default()
        };
        assert_eq!(validate_dispatch_criteria(&criteria), Ok(()));
    }

    #[test]
    fn validate_dispatch_criteria_maps_mass_out_of_range_to_domain_fault() {
        let criteria = SearchCriteria {
            taxon: "Rosa".into(),
            mass_min: -1.0,
            ..SearchCriteria::default()
        };

        assert_eq!(
            validate_dispatch_criteria(&criteria),
            Err(ValidationFault::MassOutOfRange)
        );
    }

    #[test]
    fn validate_dispatch_criteria_maps_year_range_to_domain_fault() {
        let criteria = SearchCriteria {
            taxon: "Rosa".into(),
            year_min: 2025,
            year_max: 2020,
            ..SearchCriteria::default()
        };

        assert_eq!(
            validate_dispatch_criteria(&criteria),
            Err(ValidationFault::YearRangeInvalid)
        );
    }

    #[test]
    fn validation_error_uses_typed_field_for_taxon() {
        let long = "x".repeat(501);
        let err = validate_taxon(&long).expect_err("expected taxon length error");
        assert_eq!(err.field, ValidationField::Taxon);
        assert_eq!(err.code, ValidationCode::TaxonTooLong);
    }

    #[test]
    fn validation_error_uses_typed_field_for_mass() {
        let err = validate_mass(-1.0, (-1.0, 100.0)).expect_err("expected mass range error");
        assert_eq!(err.field, ValidationField::Mass);
        assert_eq!(err.code, ValidationCode::MassOutOfRange);
    }
}
