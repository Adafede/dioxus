// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the mgf-precursor-erro-rs project

// Precursor-mass-error statistics, and the plot vocabulary they produce.
//
// Three concerns that were one file of 677 lines:
//   - [`plot`] — what the plotters draw. Nothing here knows about statistics.
//   - [`stats`] — the median machinery and the `PrecursorStats` accumulator.
//   - [`merge`] — combining two accumulators, for the two-file case.
// None of these three modules touches the browser. `plot` imports nothing at
// all; `stats` only uses `super::plot` and `std`; `merge` only uses `std`. The
// gate used to stand here on the reasoning that the plotters consuming them are
// wasm — but `plotting/` depends on `prismatica` and `std` alone and already
// compiles and tests on the host. Keeping this gate meant `cargo test` never
// compiled the median machinery, the reservoir samplers, the two-file merge or
// the accumulator, which is where the NaN and min/max defects live.
//
// Items that are genuinely browser-facing carry their own individual gates.
mod merge;
mod plot;
mod stats;

#[cfg(test)]
mod tests;

pub(crate) use merge::*;
pub(crate) use plot::*;
pub(crate) use stats::*;
