pub struct LiteratureEntry {
    pub title: &'static str,
    pub doi: &'static str,
    pub note: &'static str,
}

pub const LITERATURE: &[LiteratureEntry] = &[
    LiteratureEntry {
        title: "Natural-product-likeness score and its application to virtual screening",
        doi: "10.1186/1471-2105-13-106",
        note: "Natural-product-likeness score and fragment-driven evidence.",
    },
    LiteratureEntry {
        title: "An algorithm to identify functional groups in organic molecules",
        doi: "10.1186/s13321-017-0225-z",
        note: "Follow-up work on natural-product-inspired fragment space.",
    },
    LiteratureEntry {
        title: "The Most Common Functional Groups in Bioactive Molecules and How Their Popularity Has Evolved over Time",
        doi: "10.1021/acs.jmedchem.0c00754",
        note: "Natural-product-inspired medicinal chemistry evidence.",
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
        note: "Later benchmark / design work in NP-like libraries.",
    },
    LiteratureEntry {
        title: "Natural products in medicinal chemistry: the impact of nature's chemistry on bioactive compounds",
        doi: "10.1016/j.ejmech.2025.118178",
        note: "Recent natural-product-likeness follow-up.",
    },
    LiteratureEntry {
        title: "Ertl-style natural-product likeness and medicinal chemistry descriptor space",
        doi: "10.1021/acs.jcim.5c02538",
        note: "Additional Ertl-related evidence and descriptors.",
    },
];
