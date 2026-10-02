// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

// Precursor-mass-error statistics, and the plot vocabulary they produce.
//
// Three concerns that were one file of 677 lines:
//   - [`plot`] — what the plotters draw. Nothing here knows about statistics.
//   - [`stats`] — the median machinery and the `PrecursorStats` accumulator.
//   - [`merge`] — combining two accumulators, for the two-file case.
#[cfg(target_arch = "wasm32")]
mod merge;
mod plot;
mod stats;

#[cfg(target_arch = "wasm32")]
pub(crate) use merge::*;
pub(crate) use plot::*;
pub(crate) use stats::*;
