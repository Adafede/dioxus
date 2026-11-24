// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

mod analysis;
mod chemical_class;
mod export_and_filter;
mod format;
mod lipids;

pub(super) const EXAMPLE_MGF: &str = "\
BEGIN IONS SMILES=CCCCCCCCCCCCCCCC(=O)O PEPMASS=256.2
CHARGE=1-
TITLE=palmitic_acid
END IONS
BEGIN IONS SMILES=CC1=C(C(=CC=C1)S(=O)(=O)O)C(=O)O
CHARGE=2-
FORMULA=C9H7O4S
TITLE=non_lipid_example
END IONS
BEGIN IONS
PEPMASS=500.0
TITLE=missing_annotation
END IONS
";
