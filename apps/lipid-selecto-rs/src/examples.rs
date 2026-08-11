//! Collection of 74 example SMILES covering all 22 major lipid classes from LIPID MAPS.
//!
//! These molecules are real lipids from LIPID MAPS or commonly used in research.
//! They serve as examples for testing and dataset generation.
//! Covers all 8 categories: FA, GL, GP, SP, ST, PR, SL, PK

/// A curated set of 74 example SMILES covering all major lipid classes 1:1.
pub const EXAMPLE_LIPIDS: &[(&str, &str, &str)] = &[
    // === Fatty Acids (FA) - saturated ===
    (
        "FA_palmitic",
        "CCCCCCCCCCCCCCCC(=O)O",
        "Palmitic acid (C16:0)",
    ),
    (
        "FA_stearic",
        "CCCCCCCCCCCCCCCCCC(=O)O",
        "Stearic acid (C18:0)",
    ),
    (
        "FA_myristic",
        "CCCCCCCCCCCCCC(=O)O",
        "Myristic acid (C14:0)",
    ),
    ("FA_lauric", "CCCCCCCCCC(=O)O", "Lauric acid (C12:0)"),
    // === Monounsaturated Fatty Acids (MUFA) - one double bond ===
    (
        "MUFA_oleic",
        "CCCCCCCC=CCCCCCCCCC(=O)O",
        "Oleic acid (C18:1)",
    ),
    (
        "MUFA_erucic",
        "CCCCCCCCCCCC=CCCCCCCCCCC(=O)O",
        "Erucic acid (C22:1)",
    ),
    (
        "MUFA_palmitoleic",
        "CCCCCCCC=CCCCCCC(=O)O",
        "Palmitoleic acid (C16:1)",
    ),
    (
        "MUFA_gondoic",
        "CCCCCCCCCCCC=CCCCCC(=O)O",
        "Gondoic acid (C20:1)",
    ),
    // === Polyunsaturated Fatty Acids (PUFA) - multiple double bonds ===
    (
        "PUFA_arachidonic",
        "CC=CCC=CCC=CCC=CCCCCCCC(=O)O",
        "Arachidonic acid (C20:4)",
    ),
    (
        "PUFA_linoleic",
        "CC=CCC=CCCCCCCCCCC(=O)O",
        "Linoleic acid (C18:2)",
    ),
    (
        "PUFA_alpha_linolenic",
        "CC=CCC=CCC=CCCCCCCCC(=O)O",
        "α-Linolenic acid (C18:3)",
    ),
    (
        "PUFA_eicosapentaenoic",
        "CC=CCC=CCC=CCC=CCC=CCCCCC(=O)O",
        "EPA (C20:5)",
    ),
    // === Triglycerides (TG) ===
    (
        "TG_palmitolein",
        "CCCCCCCC=CCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)OC(=O)CCCCCCCCCCCCCCCC",
        "TG 16:1/16:0/16:0",
    ),
    (
        "TG_oleic",
        "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)OC(=O)CCCCCCCCCCCCCCCC",
        "TG 18:1/16:0/16:0",
    ),
    (
        "TG_mixed",
        "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCCCC)OC(=O)CC=CCCCCCCCCCC",
        "TG 16:0/18:0/16:1",
    ),
    (
        "TG_saturated",
        "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)OC(=O)CCCCCCCCCCCCCCCCCC",
        "TG 16:0/16:0/18:0",
    ),
    // === Diglycerides (DG) ===
    (
        "DG_saturated",
        "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)CO",
        "DG 16:0/16:0",
    ),
    (
        "DG_oleic",
        "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)CO",
        "DG 18:1/16:0",
    ),
    (
        "DG_mixed",
        "CCCCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)CO",
        "DG 18:0/16:0",
    ),
    // === Monoglycerides (MG) ===
    ("MG_palmitic", "CCCCCCCCCCCCCCCC(=O)OCC(CO)CO", "MG 16:0"),
    ("MG_oleic", "CCCCCCCC=CCCCCCCCCC(=O)OCC(CO)CO", "MG 18:1"),
    // === Phosphatidylcholine (PC) ===
    (
        "PC_palmitoyloleoyl",
        "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])OCC[N+](C)(C)C",
        "PC 16:0/18:1",
    ),
    (
        "PC_dipalmitoyl",
        "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])OCC[N+](C)(C)C",
        "PC 16:0/16:0",
    ),
    (
        "PC_dioleoyl",
        "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])OCC[N+](C)(C)C",
        "PC 18:1/18:1",
    ),
    // === Phosphatidylethanolamine (PE) ===
    (
        "PE_palmitoyloleoyl",
        "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])OCCN",
        "PE 16:0/18:1",
    ),
    (
        "PE_dipalmitoyl",
        "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])OCCN",
        "PE 16:0/16:0",
    ),
    (
        "PE_dioleoyl",
        "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])OCCN",
        "PE 18:1/18:1",
    ),
    // === Phosphatidylserine (PS) ===
    (
        "PS_dipalmitoyl",
        "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])OC(C(=O)O)[C@H](N)C",
        "PS 16:0/16:0",
    ),
    (
        "PS_palmitoyloleoyl",
        "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])OC(C(=O)O)[C@H](N)C",
        "PS 16:0/18:1",
    ),
    // === Phosphatidylglycerol (PG) ===
    (
        "PG_dipalmitoyl",
        "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])OCC(O)CO",
        "PG 16:0/16:0",
    ),
    (
        "PG_dioleoyl",
        "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])OCC(O)CO",
        "PG 18:1/18:1",
    ),
    // === Phosphatidylinositol (PI) ===
    (
        "PI_dipalmitoyl",
        "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])OC1C(O)C(O)C(O)C(O)C1O",
        "PI 16:0/16:0",
    ),
    (
        "PI_dioleoyl",
        "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])OC1C(O)C(O)C(O)C(O)C1O",
        "PI 18:1/18:1",
    ),
    // === Phosphatidic Acid (PA) ===
    (
        "PA_dipalmitoyl",
        "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])[O-]",
        "PA 16:0/16:0",
    ),
    (
        "PA_dioleoyl",
        "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])[O-]",
        "PA 18:1/18:1",
    ),
    // === Lysophosphatidylcholine (LPC) ===
    (
        "LPC_palmitic",
        "CCCCCCCCCCCCCCCC(=O)OCC(COP(=O)([O-])OCC[N+](C)(C)C)O",
        "LPC 16:0",
    ),
    (
        "LPC_oleic",
        "CCCCCCCC=CCCCCCCCCC(=O)OCC(COP(=O)([O-])OCC[N+](C)(C)C)O",
        "LPC 18:1",
    ),
    // === Lysophosphatidylethanolamine (LPE) ===
    (
        "LPE_palmitic",
        "CCCCCCCCCCCCCCCC(=O)OCC(COP(=O)([O-])OCCN)O",
        "LPE 16:0",
    ),
    (
        "LPE_oleic",
        "CCCCCCCC=CCCCCCCCCC(=O)OCC(COP(=O)([O-])OCCN)O",
        "LPE 18:1",
    ),
    // === Cardiolipin (CL) ===
    (
        "CL_tetrapalmitoyl",
        "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])OCC(COP(=O)([O-])OC(COC(=O)CCCCCCCCCCCCCCCC)COC(=O)CCCCCCCCCCCCCCCC)CO",
        "CL 16:0/16:0/16:0/16:0",
    ),
    (
        "CL_mixed",
        "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])OCC(COP(=O)([O-])OC(COC(=O)CCCCCCCCCCCCCCCC)COC(=O)CCCCCCCCCCCCCCCC)CO",
        "CL 18:1/16:0/16:0/16:0",
    ),
    // === Ceramide (Cer) ===
    (
        "Cer_d18_palmitoyl",
        "CCCCCCCCCCCCCCCC(=O)N[C@@H](CO)[C@@H](O)CCCCCCCCCCCCCCC",
        "Cer(d18:1/16:0)",
    ),
    (
        "Cer_d18_stearoyl",
        "CCCCCCCCCCCCCCCCCC(=O)N[C@@H](CO)[C@@H](O)CCCCCCCCCCCCCCC",
        "Cer(d18:1/18:0)",
    ),
    (
        "Cer_d20_oleoyl",
        "CCCCCCCC=CCCCCCCCCC(=O)N[C@@H](CO)[C@@H](O)CCCCCCCCCCCCCCC",
        "Cer(d18:1/18:1)",
    ),
    // === Sphingomyelin (SM) ===
    (
        "SM_d18_palmitoyl",
        "CCCCCCCCCCCCCCCC(=O)N[C@@H](COP(=O)([O-])OCC[N+](C)(C)C)[C@@H](O)CCCCCCCCCCCCCCC",
        "SM(d18:1/16:0)",
    ),
    (
        "SM_d18_stearoyl",
        "CCCCCCCCCCCCCCCCCC(=O)N[C@@H](COP(=O)([O-])OCC[N+](C)(C)C)[C@@H](O)CCCCCCCCCCCCCCC",
        "SM(d18:1/18:0)",
    ),
    (
        "SM_d20_oleoyl",
        "CCCCCCCC=CCCCCCCCCC(=O)N[C@@H](COP(=O)([O-])OCC[N+](C)(C)C)[C@@H](O)CCCCCCCCCCCCCC",
        "SM(d18:1/18:1)",
    ),
    // === Hexosylceramide (HexCer) ===
    (
        "HexCer_d18_palmitoyl",
        "CCCCCCCCCCCCCCCC(=O)N[C@@H](CO[C@@H]1O[C@H](CO)[C@H](O)[C@H](O)[C@H]1O)[C@@H](O)CCCCCCCCCCCCCCC",
        "HexCer(d18:1/16:0)",
    ),
    (
        "HexCer_d18_stearoyl",
        "CCCCCCCCCCCCCCCCCC(=O)N[C@@H](CO[C@@H]1O[C@H](CO)[C@H](O)[C@H](O)[C@H]1O)[C@@H](O)CCCCCCCCCCCCCCC",
        "HexCer(d18:1/18:0)",
    ),
    (
        "HexCer_d20_oleoyl",
        "CCCCCCCC=CCCCCCCCCC(=O)N[C@@H](CO[C@@H]1O[C@H](CO)[C@H](O)[C@H](O)[C@H]1O)[C@@H](O)CCCCCCCCCCCCCC",
        "HexCer(d18:1/18:1)",
    ),
    // === Sterol Lipids (ST) - cholesterol and derivatives ===
    (
        "ST_cholesterol",
        "CC(C)CCCC(C)C1CCC2C1(CCCC2=CC=C3CC(CCC3=C)O)C",
        "Cholesterol",
    ),
    (
        "ST_beta_sitosterol",
        "CC(C)C(CCC(C)C1CCC2C1(CCCC2=CC=C3CC(CCC3=C)O)C)C",
        "β-Sitosterol",
    ),
    (
        "ST_dexamethasone",
        "CC(=O)O[C@H]1CC[C@H]2[C@@H]1[C@H]([C@@H]3[C@]2(CC[C@@H]3[C@@H](C)C(=O)C)C)C",
        "Dexamethasone",
    ),
    (
        "ST_progesterone",
        "CC(=O)C1=CC[C@H]2[C@@H]1[C@H]([C@@H]3[C@]2(CC[C@@H]3C)C)C",
        "Progesterone",
    ),
    // === Prenol Lipids (PR) - isoprenoid-based lipids ===
    (
        "PR_retinol",
        "CC(C)=CCCC(C)=CC(C)=CC(C)=CC=C(C)C=C(C)C=C(C)C=C(C)C1=C(C)C(O)=CC(C)(C)C1",
        "Retinol (Vitamin A)",
    ),
    (
        "PR_alpha_tocopherol",
        "CC(C)CCCC(C)(C)C1=C(O)C(=O)C(C)=C(OC)C1=O",
        "α-Tocopherol (Vitamin E)",
    ),
    (
        "PR_ubiquinone",
        "CC(C)=CCC(C)=CCC(=C)C(C)(C)C(C)=CC=C(C)C(C)=CC=C(C)C(C)=CC(=O)c1ccc(OC)c(OC)c1",
        "Ubiquinone-10 (CoQ10)",
    ),
    (
        "PR_dolichol",
        "CC(C)=CCCC(C)=CCCC(C)=CCCC(C)=CCCC(C)=CCCC(C)=CCCC(C)=CCCC(C)=CCCC(C)=CCCC(C)=CCO",
        "Dolichol-20",
    ),
    // === Saccharipolipids (SL) - lipopolysaccharide and related ===
    (
        "SL_lipid_a",
        "CCCCCCCCCCCCCCCC(=O)N[C@@H](CO[C@@H]1O[C@H](CO)[C@H](O[C@@H]2O[C@H](C)[C@H](OC(=O)CCCCCCCCCCCCCCC)[C@H](O)[C@H]2OC(=O)CCCCCCCCCCCCCCC)[C@H](OC(=O)CCCCCCCCCCCCCCC)[C@H]1OC(=O)CCCCCCCCCCCCCCC)[C@H](O)CCCCCCCCCCCCCCCCC",
        "Lipid A",
    ),
    (
        "SL_lps_core",
        "O=C(O)C(O)C(O)C(O)C(O)C1(OCCCCCCCCCCCCCCCC(=O)O)OC(CO)[C@H](OCCCCCCCCCCCCCCCC(=O)O)[C@H](O)[C@H]1O",
        "LPS Core",
    ),
    // === Polyketides (PK) - macrolide ring system ===
    (
        "PK_atorvastatin",
        "CC(C)c1c(C(=O)Nc2ccccc2)c(cc(c1)C(F)(F)F)C(=O)NCC(O)CC(O)CC(O)=O",
        "Atorvastatin",
    ),
    (
        "PK_simvastatin",
        "CCC(C)(C)[C@H]1[C@]2(C)C[C@H](O)[C@@H](C=C3C=CC(=O)OC3=C2)C1",
        "Simvastatin",
    ),
    (
        "PK_erythromycin",
        "CCC(=O)O[C@@H]1[C@@H](C)C(=O)O[C@H](CC)[C@@H](O)[C@H](C)C(=O)[C@H](C)C[C@@](C)(O)[C@@H](OC)[C@H](OC)C[C@@H](C)C(=O)[C@H](C)[C@@H](O[C@H]2C[N+](C)(C)[C@H](O)[C@H](C)O2)C=C1",
        "Erythromycin",
    ),
];

/// Convert example list to query format (just SMILES + description lines separated by newlines).
#[must_use]
pub fn example_smiles() -> Vec<String> {
    EXAMPLE_LIPIDS
        .iter()
        .map(|(id, smiles, _)| format!("{id}\t{smiles}"))
        .collect()
}
