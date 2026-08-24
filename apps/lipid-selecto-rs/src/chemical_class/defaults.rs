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
            "#9D654C",
            "Fatty Acyls",
        ),
        ChemicalClass::new(
            "MUFA",
            "[#6][#6]=[#6][#6][#6][#6][#6][#6][#6][CX3](=[OX1])[OH]",
            "#C17754",
            "Fatty Acyls",
        ),
        ChemicalClass::new(
            "PUFA",
            "[#6][#6]=[#6][#6][#6]=[#6][#6][#6][#6][CX3](=[OX1])[OH]",
            "#F09163",
            "Fatty Acyls",
        ),
    ]
}

pub(super) fn glycerolipids() -> Vec<ChemicalClass> {
    vec![
        ChemicalClass::new(
            "TG(AAA)",
            "[CX4]([OX2][CX3](=[OX1])[#6])([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]",
            "#098BD9",
            "Glycerolipids",
        ),
        ChemicalClass::new(
            "DG(AA)",
            "[CX4]([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]",
            "#56B4E9",
            "Glycerolipids",
        ),
        ChemicalClass::new(
            "MG(A)",
            "[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6]",
            "#7DCCFF",
            "Glycerolipids",
        ),
    ]
}

pub(super) fn glycerophospholipids() -> Vec<ChemicalClass> {
    vec![
        ChemicalClass::new(
            "PC(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[NX4+]([CH3])([CH3])[CH3]",
            "#4E7705",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "PE(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CH2X4][NX3;H2,H1,H0]",
            "#6D9F06",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "PS(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([CX3](=[OX1])[OX2H,OX1-])[NX3]",
            "#97CE2F",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "PI(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[C;R1]1[CH;R1][CH;R1][CH;R1][CH;R1][CH;R1]1",
            "#DDFFA0",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "PG(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2H,OX1-])[CH2X4][OX2H,OX1-]",
            "#BDEC6F",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "PA(AA)",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4][CH2X4][OX2H,OX1-]",
            "#DDFFA0",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "LPC(A)",
            "[CH2X4][CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2][PX4](=[OX1])[OX2][CH2X4][N+;X4]",
            "#148F77",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "LPE(A)",
            "[CH2X4][CHX4]([OX2][CX3](=[OX1])[#6])[CH2X4][OX2][PX4](=[OX1])[OX2][CH2X4][NX3]",
            "#009E73",
            "Glycerophospholipids",
        ),
        ChemicalClass::new(
            "CL(AAAA)",
            "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2])[CH2X4][OX2]",
            "#43BA8F",
            "Glycerophospholipids",
        ),
    ]
}

pub(super) fn sphingolipids() -> Vec<ChemicalClass> {
    vec![
        ChemicalClass::new(
            "Cer(AS)",
            "[NX3][CX3](=[OX1])[CX4]",
            "#7D3560",
            "Sphingolipids",
        ),
        ChemicalClass::new(
            "SM(AS)",
            "[NX4+][CX4][CX4][OX2][PX4](=[OX1])[OX2]",
            "#A1527F",
            "Sphingolipids",
        ),
        ChemicalClass::new(
            "HexCer(AS)",
            "[NX3][CX3](=[OX1])[CX4][CH1X4][CH1X4][OX2][CH1X4][CH1X4]",
            "#CC79A7",
            "Sphingolipids",
        ),
    ]
}

pub(super) fn sterol_lipids() -> Vec<ChemicalClass> {
    vec![ChemicalClass::new(
        "ST",
        "[#6]1[#6][#6][#6]2[#6]([#6]1)[#6][#6][#6]2([#6])[#6]",
        "#6a51a3",
        "Sterol Lipids",
    )]
}

pub(super) fn prenol_lipids() -> Vec<ChemicalClass> {
    vec![ChemicalClass::new(
        "PR",
        "[#6]=[#6][#6]=[#6][#6]",
        "#ff7f00",
        "Prenol Lipids",
    )]
}

pub(super) fn saccharolipids() -> Vec<ChemicalClass> {
    vec![ChemicalClass::new(
        "SL",
        "[#6][OX2][PX4](=[OX1])[OX2][#6]",
        "#4292c6",
        "Saccharolipids",
    )]
}

pub(super) fn polyketides() -> Vec<ChemicalClass> {
    vec![ChemicalClass::new(
        "PK",
        "[#6;R]1[#6]([#6](=[OX1])[#6])[#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R][#6;R]1",
        "#238b45",
        "Polyketides",
    )]
}
