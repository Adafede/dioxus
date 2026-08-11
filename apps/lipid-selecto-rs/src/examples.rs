//! Collection of 50 example SMILES covering all 18 major lipid classes.
//!
//! These molecules are real lipids from LIPID MAPS or commonly used in research.
//! They serve as examples for testing and dataset generation.
//! All 18 classes are covered: FA, MUFA, PUFA, TG, DG, MG, PC, PE, PS, PG, PI, PA, LPC, LPE, CL, Cer, SM, HexCer

/// A curated set of 50 example SMILES covering all major lipid classes 1:1.
pub const EXAMPLE_LIPIDS: &[(&str, &str, &str)] = &[
    // === Fatty Acids (FA) ===
    ("FA_palmitic", "CCCCCCCCCCCCCCCC(=O)O", "Palmitic acid (C16:0)"),
    ("FA_stearic", "CCCCCCCCCCCCCCCCCC(=O)O", "Stearic acid (C18:0)"),
    ("FA_myristic", "CCCCCCCCCCCCCC(=O)O", "Myristic acid (C14:0)"),
    ("FA_lauric", "CCCCCCCCCC(=O)O", "Lauric acid (C12:0)"),
    
    // === Monounsaturated Fatty Acids (MUFA) ===
    ("MUFA_oleic", "CCCCCCCC=CCCCCCCCCC(=O)O", "Oleic acid (C18:1)"),
    ("MUFA_erucic", "CCCCCCCCCCCC=CCCCCCCCCCC(=O)O", "Erucic acid (C22:1)"),
    ("MUFA_palmitoleic", "CCCCCCCC=CCCCCCC(=O)O", "Palmitoleic acid (C16:1)"),
    ("MUFA_gondoic", "CCCCCCCCCCCC=CCCCCC(=O)O", "Gondoic acid (C20:1)"),
    
    // === Polyunsaturated Fatty Acids (PUFA) ===
    ("PUFA_arachidonic", "CC=CCC=CCC=CCC=CCCCCCCC(=O)O", "Arachidonic acid (C20:4)"),
    ("PUFA_linoleic", "CC=CCC=CCCCCCCCCCC(=O)O", "Linoleic acid (C18:2)"),
    ("PUFA_alpha_linolenic", "CC=CCC=CCC=CCCCCCCCC(=O)O", "α-Linolenic acid (C18:3)"),
    ("PUFA_eicosapentaenoic", "CC=CCC=CCC=CCC=CCC=CCCCCC(=O)O", "EPA (C20:5)"),
    
    // === Triglycerides (TG) ===
    ("TG_palmitolein", "CCCCCCCC=CCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)OC(=O)CCCCCCCCCCCCCCCC", "TG 16:1/16:0/16:0"),
    ("TG_oleic", "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)OC(=O)CCCCCCCCCCCCCCCC", "TG 18:1/16:0/16:0"),
    ("TG_mixed", "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCCCC)OC(=O)CC=CCCCCCCCCCC", "TG 16:0/18:0/16:1"),
    ("TG_saturated", "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)OC(=O)CCCCCCCCCCCCCCCCCC", "TG 16:0/16:0/18:0"),
    
    // === Diglycerides (DG) ===
    ("DG_saturated", "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)CO", "DG 16:0/16:0"),
    ("DG_oleic", "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)CO", "DG 18:1/16:0"),
    ("DG_mixed", "CCCCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)CO", "DG 18:0/16:0"),
    
    // === Monoglycerides (MG) ===
    ("MG_palmitic", "CCCCCCCCCCCCCCCC(=O)OCC(CO)CO", "MG 16:0"),
    ("MG_oleic", "CCCCCCCC=CCCCCCCCCC(=O)OCC(CO)CO", "MG 18:1"),
    
    // === Phosphatidylcholine (PC) ===
    ("PC_palmitoyloleoyl", "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])OCC[N+](C)(C)C", "PC 16:0/18:1"),
    ("PC_dipalmitoyl", "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])OCC[N+](C)(C)C", "PC 16:0/16:0"),
    ("PC_dioleoyl", "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])OCC[N+](C)(C)C", "PC 18:1/18:1"),
    
    // === Phosphatidylethanolamine (PE) ===
    ("PE_palmitoyloleoyl", "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])OCCN", "PE 16:0/18:1"),
    ("PE_dipalmitoyl", "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])OCCN", "PE 16:0/16:0"),
    ("PE_dioleoyl", "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])OCCN", "PE 18:1/18:1"),
    
    // === Phosphatidylserine (PS) ===
    ("PS_dipalmitoyl", "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])OC(C(=O)O)[C@H](N)C", "PS 16:0/16:0"),
    ("PS_palmitoyloleoyl", "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])OC(C(=O)O)[C@H](N)C", "PS 16:0/18:1"),
    
    // === Phosphatidylglycerol (PG) ===
    ("PG_dipalmitoyl", "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])OCC(O)CO", "PG 16:0/16:0"),
    ("PG_dioleoyl", "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])OCC(O)CO", "PG 18:1/18:1"),
    
    // === Phosphatidylinositol (PI) ===
    ("PI_dipalmitoyl", "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])OC1C(O)C(O)C(O)C(O)C1O", "PI 16:0/16:0"),
    ("PI_dioleoyl", "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])OC1C(O)C(O)C(O)C(O)C1O", "PI 18:1/18:1"),
    
    // === Phosphatidic Acid (PA) ===
    ("PA_dipalmitoyl", "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])[O-]", "PA 16:0/16:0"),
    ("PA_dioleoyl", "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCC=CCCCCCCCCC)COP(=O)([O-])[O-]", "PA 18:1/18:1"),
    
    // === Lysophosphatidylcholine (LPC) ===
    ("LPC_palmitic", "CCCCCCCCCCCCCCCC(=O)OCC(COP(=O)([O-])OCC[N+](C)(C)C)O", "LPC 16:0"),
    ("LPC_oleic", "CCCCCCCC=CCCCCCCCCC(=O)OCC(COP(=O)([O-])OCC[N+](C)(C)C)O", "LPC 18:1"),
    
    // === Lysophosphatidylethanolamine (LPE) ===
    ("LPE_palmitic", "CCCCCCCCCCCCCCCC(=O)OCC(COP(=O)([O-])OCCN)O", "LPE 16:0"),
    ("LPE_oleic", "CCCCCCCC=CCCCCCCCCC(=O)OCC(COP(=O)([O-])OCCN)O", "LPE 18:1"),
    
    // === Cardiolipin (CL) ===
    ("CL_tetrapalmitoyl", "CCCCCCCCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])OCC(COP(=O)([O-])OC(COC(=O)CCCCCCCCCCCCCCCC)COC(=O)CCCCCCCCCCCCCCCC)CO", "CL 16:0/16:0/16:0/16:0"),
    ("CL_mixed", "CCCCCCCC=CCCCCCCCCC(=O)OC(COC(=O)CCCCCCCCCCCCCCCC)COP(=O)([O-])OCC(COP(=O)([O-])OC(COC(=O)CCCCCCCCCCCCCCCC)COC(=O)CCCCCCCCCCCCCCCC)CO", "CL 18:1/16:0/16:0/16:0"),
    
    // === Ceramide (Cer) ===
    ("Cer_d18_palmitoyl", "CCCCCCCCCCCCCCCC(=O)N[C@@H](CO)[C@@H](O)CCCCCCCCCCCCCCC", "Cer(d18:1/16:0)"),
    ("Cer_d18_stearoyl", "CCCCCCCCCCCCCCCCCC(=O)N[C@@H](CO)[C@@H](O)CCCCCCCCCCCCCCC", "Cer(d18:1/18:0)"),
    ("Cer_d20_oleoyl", "CCCCCCCC=CCCCCCCCCC(=O)N[C@@H](CO)[C@@H](O)CCCCCCCCCCCCCCC", "Cer(d18:1/18:1)"),
    
    // === Sphingomyelin (SM) ===
    ("SM_d18_palmitoyl", "CCCCCCCCCCCCCCCC(=O)N[C@@H](COP(=O)([O-])OCC[N+](C)(C)C)[C@@H](O)CCCCCCCCCCCCCCC", "SM(d18:1/16:0)"),
    ("SM_d18_stearoyl", "CCCCCCCCCCCCCCCCCC(=O)N[C@@H](COP(=O)([O-])OCC[N+](C)(C)C)[C@@H](O)CCCCCCCCCCCCCCC", "SM(d18:1/18:0)"),
    ("SM_d20_oleoyl", "CCCCCCCC=CCCCCCCCCC(=O)N[C@@H](COP(=O)([O-])OCC[N+](C)(C)C)[C@@H](O)CCCCCCCCCCCCCC", "SM(d18:1/18:1)"),
    
    // === Hexosylceramide (HexCer) ===
    ("HexCer_d18_palmitoyl", "CCCCCCCCCCCCCCCC(=O)N[C@@H](CO[C@@H]1O[C@H](CO)[C@H](O)[C@H](O)[C@H]1O)[C@@H](O)CCCCCCCCCCCCCCC", "HexCer(d18:1/16:0)"),
    ("HexCer_d18_stearoyl", "CCCCCCCCCCCCCCCCCC(=O)N[C@@H](CO[C@@H]1O[C@H](CO)[C@H](O)[C@H](O)[C@H]1O)[C@@H](O)CCCCCCCCCCCCCCC", "HexCer(d18:1/18:0)"),
    ("HexCer_d20_oleoyl", "CCCCCCCC=CCCCCCCCCC(=O)N[C@@H](CO[C@@H]1O[C@H](CO)[C@H](O)[C@H](O)[C@H]1O)[C@@H](O)CCCCCCCCCCCCCC", "HexCer(d18:1/18:1)"),
];

/// Convert example list to query format (just SMILES + description lines separated by newlines).
pub fn example_smiles() -> Vec<String> {
    EXAMPLE_LIPIDS
        .iter()
        .map(|(id, smiles, _)| format!("{}\t{}", id, smiles))
        .collect()
}
