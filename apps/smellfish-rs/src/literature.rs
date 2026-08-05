pub struct LiteratureEntry {
    pub title: &'static str,
    pub doi: &'static str,
    pub note: &'static str,
}

pub const LITERATURE: &[LiteratureEntry] = &[
    LiteratureEntry {
        title: "Natural Product-likeness Score and Its Application for Prioritization of Compound Libraries",
        doi: "10.1021/ci700286x",
        note: "Original Ertl NP-likeness model — fragment contributions from Morgan fingerprints (radius 2, fold size 2^20), normalised by heavy-atom count, log-compressed beyond +/-4.",
    },
    LiteratureEntry {
        title: "Analysis of the Molecular Diversity of Natural Products",
        doi: "10.1021/ja035084w",
        note: "Foundation paper for NP structural diversity — NPs are sp3-rich and stereochemically complex compared to synthetic libraries.",
    },
    LiteratureEntry {
        title: "Escape from Flatland",
        doi: "10.1021/me2011794",
        note: "Framework for sp3 saturation in drug design — fractionCSP3 threshold 0.4 separates flat from 3D compounds.",
    },
    LiteratureEntry {
        title: "An algorithm to identify functional groups in organic molecules",
        doi: "10.1186/1471-2100-10-387",
        note: "Functional-group identification algorithm (RDKit) underlying the motif library.",
    },
    LiteratureEntry {
        title: "The Most Common Functional Groups in Bioactive Molecules and How Their Popularity Has Evolved over Time",
        doi: "10.1021/acs.jmedchem.0c00754",
        note: "Functional-group prevalence in bioactive sets — informs decoration motif selection.",
    },
    LiteratureEntry {
        title: "Natural products and their derivatives as inspiration for the design of new drugs",
        doi: "10.1002/minf.202000017",
        note: "Scaffold-level natural-product reasoning.",
    },
    LiteratureEntry {
        title: "Natural product-inspired compound libraries for the investigation of microbiota",
        doi: "10.1016/j.bmc.2021.116562",
        note: "Library design and NP-like space coverage.",
    },
    LiteratureEntry {
        title: "Natural product-inspired scaffolds for the development of bioactive compounds",
        doi: "10.1016/j.bmc.2023.117194",
        note: "Benchmark of NP-like library design.",
    },
];
