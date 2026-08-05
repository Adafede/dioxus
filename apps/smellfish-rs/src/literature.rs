#[allow(dead_code)]
pub struct LiteratureEntry {
    pub title: &'static str,
    pub doi: &'static str,
    pub note: &'static str,
}

/// All DOIs below were verified via Crossref as of 2026-08-05.
pub const LITERATURE: &[LiteratureEntry] = &[
    LiteratureEntry {
        title: "Natural Product-likeness Score and Its Application for Prioritization of Compound Libraries",
        doi: "10.1021/ci700286x",
        note: "Ertl, Roggo & Schuffenhauer (2008). Original NP-likeness model — fragment contributions from Morgan fingerprints (radius 2), normalised by heavy-atom count, log-compressed beyond +/-4.",
    },
    LiteratureEntry {
        title: "Cheminformatic Analysis of Natural Products and their Chemical Space",
        doi: "10.2533/chimia.2007.355",
        note: "Wetzel, Schuffenhauer, Roggo, Ertl & Waldmann (2007). NPs are sp3-rich — 57% sp3 carbons vs 42% in drugs.",
    },
    LiteratureEntry {
        title: "Analysis of the Natural-Product Content in Commercial Screening Collections: The Impact of a Natural-Product Clone Database",
        doi: "10.1021/acs.jnatprod.8b01022",
        note: "Ertl & Schuhmann (2019, J. Nat. Prod.). Functional-group and motif catalogue enriched in natural products — used to prioritise the motif library and the top-60 substituent patterns.",
    },
    LiteratureEntry {
        title: "Ring systems in medicinal chemistry: A cheminformatics analysis of ring popularity in drug discovery over time",
        doi: "10.1016/j.ejmech.2025.118178",
        note: "Ertl, Altmann & Wilcken (2025). Ring-system prevalence — distinguishes NP from synthetic ring architectures.",
    },
    LiteratureEntry {
        title: "An algorithm to identify functional groups in organic molecules",
        doi: "10.1186/s13321-017-0225-z",
        note: "RDKit functional-group detection algorithm — powers the motif library.",
    },
    LiteratureEntry {
        title: "The Most Common Functional Groups in Bioactive Molecules and How Their Popularity Has Evolved over Time",
        doi: "10.1021/acs.jmedchem.0c00754",
        note: "Functional-group prevalence in bioactive sets — informs decoration motif selection.",
    },
    LiteratureEntry {
        title: "Cheminformatics Analysis of Natural Product Scaffolds: Comparison of Scaffolds Produced by Animals, Plants, Fungi and Bacteria",
        doi: "10.1002/minf.202000017",
        note: "Scaffold-level natural-product reasoning — NP scaffolds differ by biosynthetic origin (animals, plants, fungi, bacteria).",
    },
    LiteratureEntry {
        title: "Substituents of life: The most common substituent patterns present in natural products",
        doi: "10.1016/j.bmc.2021.116562",
        note: "Ertl (2022, BMC 54, 116562). Top-60 most common natural-product substituents — matched via RDKit substructure search to detect biosynthetic fingerprint.",
    },
    LiteratureEntry {
        title: "The most common linkers in bioactive molecules and their bioisosteric replacement network",
        doi: "10.1016/j.bmc.2023.117194",
        note: "Linker prevalence in bioactive molecules — NP linkers differ from synthetic ones.",
    },
];
