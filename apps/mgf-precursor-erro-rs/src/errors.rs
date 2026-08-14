//! Typed error type for `mgf-precursor-erro-rs`.
//!
//! Replaces the bare `.unwrap()` panics on plotters drawing back-end operations
//! (`fill` / `present` / `draw`) with explicit `Result` propagation via
//! [`MgfError`] and the `?` operator, matching the typed-error convention used
//! by `crates/lotus` and `crates/upload`.
use std::fmt;

use plotters::drawing::DrawingAreaErrorKind;

/// Errors that can occur while rendering diagnostic SVGs for
/// `mgf-precursor-erro-rs`.
#[derive(Debug)]
pub enum MgfError {
    /// A plotters drawing back-end operation (`fill`/`present`/`draw`) failed.
    Drawing(String),
}

impl fmt::Display for MgfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MgfError::Drawing(msg) => write!(f, "SVG rendering failed: {msg}"),
        }
    }
}

impl std::error::Error for MgfError {}

/// Convert any plotters drawing-area error into [`MgfError`] so the `render_*`
/// helpers can propagate failures with `?` instead of panicking.
impl<E> From<DrawingAreaErrorKind<E>> for MgfError
where
    E: std::error::Error + Send + Sync,
{
    fn from(error: DrawingAreaErrorKind<E>) -> Self {
        MgfError::Drawing(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_renders_message() {
        let err = MgfError::Drawing("bad layout".to_string());
        assert_eq!(format!("{err}"), "SVG rendering failed: bad layout");
    }
}
