// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lipid-selecto-rs project

//! Broad-grained lipid class definitions (one SMARTS pattern per LIPID MAPS family).
//!
//! These are the "defaults" — coarse patterns covering the major lipid families.
//! They are primarily used for color attribution and simple lipid/non-lipid
//! detection. For precise LMSD-subclass-level classification, see [`super::lmsd`].

use super::ChemicalClass;

pub(super) fn fatty_acyls() -> Vec<ChemicalClass> {
    vec![
        ChemicalClass::new(
            "FA",
            "[#6][#6][#6][#6][#6][#6][#6][#6][CX3](=[OX1])[OH]",
            super::FA_PALETTE[0],
            "Fatty Acyls",
        ),
        ChemicalClass::new(
            "MUFA",
            "[#6][#6]=[#6][#6][#6][#6][#6][#6][#6][CX3](=[OX1])[OH]",
            super::FA_PALETTE[1],
            "Fatty Acyls",
        ),
        ChemicalClass::new(
            "PUFA",
            "[#6][#6]=[#6][#6][#6]=[#6][#6][#6][#6][CX3](=[OX1])[OH]",
            super::FA_PALETTE[2],
            "Fatty Acyls",
        ),
    ]
}

pub(super) fn glycerolipids() -> Vec<ChemicalClass> {
    vec![
        ChemicalClass::new(
            "TG(AAA)",
            "[CX4]([OX2][CX3](=[OX1])[#6])([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]",
            super::GL_PALETTE[0],
            "Glycerolipids",
        ),
        ChemicalClass::new(
            "DG(AA)",
            "[CX4]([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]",
            super::GL_PALETTE[1],
            "Glycerolipids",
        ),
        ChemicalClass::new(
            "MG(A)",
            "[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6]",
            super::GL_PALETTE[2],
            "Glycerolipids",
        ),
    ]
}

pub(super) fn glycerophospholipids() -> Vec<ChemicalClass> {
    vec![
        ChemicalClass::new(
            "PC(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[NX4+]([CH3])([CH3])[CH3]",
            super::GP_PALETTE[0],
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "PE(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CH2X4][NX3;H2,H1,H0]",
            super::GP_PALETTE[1],
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "PS(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([CX3](=[OX1])[OX2H,OX1-])[NX3]",
            super::GP_PALETTE[2],
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "PI(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[C;R1]1[CH;R1][CH;R1][CH;R1][CH;R1][CH;R1]1",
            super::GP_PALETTE[3],
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "PG(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2H,OX1-])[CH2X4][OX2H,OX1-]",
            super::GP_PALETTE[4],
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "PA(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4][CH2X4][OX2H,OX1-]",
            super::GP_PALETTE[4],
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "LPC(A)",
            "[CH2X4][CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2][PX4](=[OX1])[OX2][CH2X4][N+;X4]",
            super::GP_PALETTE[0],
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "LPE(A)",
            "[CH2X4][CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2][PX4](=[OX1])[OX2][CH2X4][NX3]",
            super::GP_PALETTE[1],
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "CL(AAAA)",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2])[CH2X4][OX2]",
            super::GP_PALETTE[2],
            "Glycerophospholipids",
        ),
    ]
}

pub(super) fn sphingolipids() -> Vec<ChemicalClass> {
    vec![
        ChemicalClass::new(
            "Cer(AS)",
            "[NX3][CX3](=[OX1])[CX4]",
            super::SP_PALETTE[0],
            "Sphingolipids",
        ),
        ChemicalClass::new(
            "SM(AS)",
            "[NX4+][CX4][CX4][OX2][PX4](=[OX1])[OX2]",
            super::SP_PALETTE[1],
            "Sphingolipids",
        ),
        ChemicalClass::new(
            "HexCer(AS)",
            "[NX3][CX3](=[OX1])[CX4][CH1X4][CH1X4][OX2][CH1X4][CH1X4]",
            super::SP_PALETTE[2],
            "Sphingolipids",
        ),
    ]
}

pub(super) fn sterol_lipids() -> Vec<ChemicalClass> {
    vec![ChemicalClass::new(
        "ST",
        "[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6][#6]2([#6])[#6]",
        super::ST_PALETTE[0],
        "Sterol Lipids",
    )]
}

pub(super) fn prenol_lipids() -> Vec<ChemicalClass> {
    vec![ChemicalClass::new(
        "PR",
        "[#6]=[#6][#6]=[#6][#6]",
        super::PR_PALETTE[0],
        "Prenol Lipids",
    )]
}

pub(super) fn saccharolipids() -> Vec<ChemicalClass> {
    vec![ChemicalClass::new(
        "SL",
        "[#6][OX2][PX4](=[OX1])[OX2][#6]",
        super::SL_PALETTE[0],
        "Saccharolipids",
    )]
}

pub(super) fn polyketides() -> Vec<ChemicalClass> {
    vec![ChemicalClass::new(
        "PK",
        "[#6;R]1[#6]([#6](=[OX1])[#6])[#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R]1",
        super::PK_PALETTE[0],
        "Polyketides",
    )]
}
