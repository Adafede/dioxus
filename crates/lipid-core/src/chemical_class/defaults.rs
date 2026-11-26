// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Coarse-grained default lipid SMARTS rules used by unit characterization tests.

use super::ChemicalClass;
use crate::family::{
    FA_PALETTE, GL_PALETTE, GP_PALETTE, LipidFamily, PK_PALETTE, PR_PALETTE, SL_PALETTE,
    SP_PALETTE, ST_PALETTE,
};

const ENTRIES: &[(&str, &str, &str, LipidFamily)] = &[
    (
        "FA",
        "[#6][#6][#6][#6][#6][#6][#6][#6][CX3](=[OX1])[OH]",
        FA_PALETTE[0],
        LipidFamily::FattyAcyls,
    ),
    (
        "MUFA",
        "[#6][#6]=[#6][#6][#6][#6][#6][#6][#6][CX3](=[OX1])[OH]",
        FA_PALETTE[1],
        LipidFamily::FattyAcyls,
    ),
    (
        "PUFA",
        "[#6][#6]=[#6][#6][#6]=[#6][#6][#6][#6][CX3](=[OX1])[OH]",
        FA_PALETTE[2],
        LipidFamily::FattyAcyls,
    ),
    (
        "TG(AAA)",
        "[CH2X4]([OX2][CX3](=[OX1])[#6])[CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2][CX3](=[OX1])[#6]",
        GL_PALETTE[0],
        LipidFamily::Glycerolipids,
    ),
    (
        "DG(AA)",
        "[CH2X4]([OX2][CX3](=[OX1])[#6])[CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2H]",
        GL_PALETTE[1],
        LipidFamily::Glycerolipids,
    ),
    (
        "MG(A)",
        "[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6]",
        GL_PALETTE[2],
        LipidFamily::Glycerolipids,
    ),
    (
        "PC(AA)",
        "[CH2X4][CHX4]([OX2])[CH2X4][OX2][PX4](=[OX1])([OX2H,OX1-])[OX2][CH2X4][CH2X4][NX4+]([CH3])([CH3])[CH3]",
        GP_PALETTE[0],
        LipidFamily::Glycerophospholipids,
    ),
    (
        "PE(AA)",
        "[PX4](=[OX1])([OX2H,OX1-])([OX2])[OX2][CH2X4][CH2X4][NX3;H2,H1,H0]",
        GP_PALETTE[1],
        LipidFamily::Glycerophospholipids,
    ),
    (
        "PS(AA)",
        "[PX4](=[OX1])([OX2H,OX1-])([OX2])[OX2][CH2X4][CHX4]([CX3](=[OX1])[OX2H,OX1-])[NX3]",
        GP_PALETTE[2],
        LipidFamily::Glycerophospholipids,
    ),
    (
        "PI(AA)",
        "[PX4](=[OX1])([OX2H,OX1-])([OX2])[OX2][C;R1]1[CH;R1]([OX2])[CH;R1]([OX2])[CH;R1]([OX2])[CH;R1]([OX2])[CH;R1]1[OX2]",
        GP_PALETTE[3],
        LipidFamily::Glycerophospholipids,
    ),
    (
        "PG(AA)",
        "[PX4](=[OX1])([OX2H,OX1-])([OX2])[OX2][CH2X4][CHX4]([OX2H,OX1-])[CH2X4][OX2H,OX1-]",
        GP_PALETTE[4],
        LipidFamily::Glycerophospholipids,
    ),
    (
        "PA(AA)",
        "[CH2X4][CHX4]([OX2])[CH2X4][OX2][PX4](=[OX1])([OX2H,OX1-])[OX2H,OX1-]",
        GP_PALETTE[4],
        LipidFamily::Glycerophospholipids,
    ),
    (
        "LPC(A)",
        "[CH2X4][CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2][PX4](=[OX1])[OX2][CH2X4][N+;X4]",
        GP_PALETTE[0],
        LipidFamily::Glycerophospholipids,
    ),
    (
        "LPE(A)",
        "[CH2X4][CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2][PX4](=[OX1])[OX2][CH2X4][NX3]",
        GP_PALETTE[1],
        LipidFamily::Glycerophospholipids,
    ),
    (
        "CL(AAAA)",
        "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2])[CH2X4][OX2]",
        GP_PALETTE[2],
        LipidFamily::Glycerophospholipids,
    ),
    (
        "Cer(AS)",
        "[NX3][CX3](=[OX1])[CX4]",
        SP_PALETTE[0],
        LipidFamily::Sphingolipids,
    ),
    (
        "SM(AS)",
        "[NX4+][CX4][CX4][OX2][PX4](=[OX1])[OX2]",
        SP_PALETTE[1],
        LipidFamily::Sphingolipids,
    ),
    (
        "HexCer(AS)",
        "[NX3][CX3](=[OX1])[CX4][CH1X4][CH1X4][OX2][CH1X4][CH1X4]",
        SP_PALETTE[2],
        LipidFamily::Sphingolipids,
    ),
    (
        "ST",
        "[#6]1~[#6]~[#6]~[#6]2~[#6](~[#6]~1)~[#6]~[#6]~[#6]3~[#6]~2~[#6]~[#6]~[#6]4~[#6]~3~[#6]~[#6]~[#6]~4",
        ST_PALETTE[0],
        LipidFamily::SterolLipids,
    ),
    (
        "PR",
        "[#6]=[#6][#6]=[#6][#6]",
        PR_PALETTE[0],
        LipidFamily::PrenolLipids,
    ),
    (
        "SL",
        "[CX3](=[OX1])[NX3][CHX4]1[CHX4]([OX2])[CHX4]([OX2])[CHX4]([OX2])[CHX4]([CH2X4][OX2])[OX2]1",
        SL_PALETTE[0],
        LipidFamily::Saccharolipids,
    ),
    (
        "PK",
        "[#6;R]1[#6]([#6](=[OX1])[#6])[#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R]1",
        PK_PALETTE[0],
        LipidFamily::Polyketides,
    ),
];

pub(super) fn all() -> Vec<ChemicalClass> {
    ENTRIES
        .iter()
        .enumerate()
        .map(|(index, &(name, smarts, color, family))| {
            ChemicalClass::new(index, name, smarts, color, family)
        })
        .collect()
}
