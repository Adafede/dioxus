// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! CLI entry point for the `smarts-evoliposuction` binary.

mod cli;
#[cfg(test)]
mod cli_tests;

use clap::Parser;
use smarts_evoliposuction::CliError;

fn main() -> Result<(), CliError> {
    cli::run(cli::Args::parse())
}
