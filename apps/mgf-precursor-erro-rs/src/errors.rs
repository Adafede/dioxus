// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

//! Typed SVG plotting error definitions for `mgf-precursor-erro-rs`.

use plotters::drawing::DrawingAreaErrorKind;

/// Errors raised while rendering diagnostic or scatter SVGs.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum PlotError {
    /// A plotters drawing back-end operation (`fill`, `draw`, or `present`) failed.
    #[error("SVG rendering failed: {0}")]
    Drawing(String),
}

impl<E> From<DrawingAreaErrorKind<E>> for PlotError
where
    E: std::error::Error + Send + Sync,
{
    fn from(error: DrawingAreaErrorKind<E>) -> Self {
        Self::Drawing(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::PlotError;

    #[test]
    fn display_renders_message() {
        let err = PlotError::Drawing("bad layout".to_string());
        assert_eq!(err.to_string(), "SVG rendering failed: bad layout");
    }
}
