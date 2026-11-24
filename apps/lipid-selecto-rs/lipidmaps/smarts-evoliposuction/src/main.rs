// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! CLI entry point for the `smarts-evoliposuction` binary.

#[expect(
    clippy::redundant_pub_crate,
    reason = "contradicts rustc unreachable_pub = deny on binary submodules"
)]
mod cli;
#[cfg(test)]
mod cli_tests;

use clap::Parser;
use smarts_evoliposuction::CliError;

fn main() -> Result<(), CliError> {
    cli::run(cli::Args::parse())
}
