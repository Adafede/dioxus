// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Inlined from `crates/lipidsdl` — generic download, SDF parsing, and lipid-data conversion utilities.

#[cfg(not(target_arch = "wasm32"))]
pub mod download;
pub mod sdf;
