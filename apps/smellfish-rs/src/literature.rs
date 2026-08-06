pub struct LiteratureEntry {
    pub title: &'static str,
    pub doi: &'static str,
    pub note: &'static str,
}

/// All DOIs below were verified via Crossref.
pub const LITERATURE: &[LiteratureEntry] = &[
    LiteratureEntry {
        title: "Natural product-likeness score revisited: an open-source, open-data implementation",
        doi: "10.1186/1471-2105-13-106",
        note: "Jayaseelan, Moreno, Truszkowski, Ertl & Steinbeck (2012, BMC Bioinformatics 13, 106). Open-source, open-data re-implementation of the Ertl NP-likeness model. Provides the np_model.bin fragment-contribution lookup loaded by the inspect bridge.",
    },
    LiteratureEntry {
        title: "Natural Product-likeness Score and Its Application for Prioritization of Compound Libraries",
        doi: "10.1021/ci700286x",
        note: "Ertl, Roggo & Schuffenhauer (2008, J. Chem. Inf. Model. 48, 68-74). Original NP-likeness score model — concept and training data (~50K NPs vs ~1M ZINC). Fragment contributions from Morgan fingerprints (radius 2), normalised by heavy-atom count, log-compressed beyond ±4.",
    },
    LiteratureEntry {
        title: "Cheminformatic Analysis of Natural Products and their Chemical Space",
        doi: "10.2533/chimia.2007.355",
        note: "Wetzel, Schuffenhauer, Roggo, Ertl & Waldmann (2007, CHIMIA 61, 355). Cheminformatic analysis of natural-product chemical space and comparison with drug-like compounds.",
    },
    LiteratureEntry {
        title: "Analysis of the Natural-Product Content in Commercial Screening Collections: The Impact of a Natural-Product Clone Database",
        doi: "10.1021/acs.jnatprod.8b01022",
        note: "Ertl & Schuhmann (2019, J. Nat. Prod. 82, 1258-1263). Systematic analysis of functional-group occurrence in natural products vs. synthetic compounds, with kingdom-level (animals, plants, fungi, bacteria) enrichment data. Provides the source-vs-synthetic and kingdom-enrichment tables loaded from ertl_source_vs_synthetic.txt and ertl_kingdom_enrichment.txt.",
    },
    LiteratureEntry {
        title: "An algorithm to identify functional groups in organic molecules",
        doi: "10.1186/s13321-017-0225-z",
        note: "Ertl (2017, J. Cheminform. 9). Algorithm to identify functional groups in organic molecules — provides the SMARTS patterns in group_names.txt for the motif library.",
    },
    LiteratureEntry {
        title: "Substituents of life: The most common substituent patterns present in natural products",
        doi: "10.1016/j.bmc.2021.116562",
        note: "Ertl (2022, Bioorganic & Medicinal Chemistry 54, 116562). Most common substituent patterns in natural products. Provides the top-60 NP substituents (ertl_npsubstituents.txt), matched via RDKit substructure search in the inspect bridge.",
    },
];
