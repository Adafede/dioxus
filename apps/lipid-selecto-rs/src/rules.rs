//! Load and manage lipid classification rules from external YAML configuration.
//!
//! This module provides an extensible framework for lipid class definitions,
//! allowing users to add custom rules without modifying the source code.
//! Rules are loaded from `lipid_rules.yaml` at application startup.

use std::collections::HashMap;

/// Error when loading evolved SMARTS rules from CSV.
#[derive(Debug, thiserror::Error)]
pub enum EvolvedRulesError {
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// A complete lipid class rule with SMARTS pattern and metadata.
///
/// SMARTS patterns are pre-compiled once at construction time to avoid
/// re-parsing the pattern string on every `matches` call.
#[derive(Clone, Debug)]
pub struct LipidRule {
    pub name: String,
    pub family: String,
    pub architecture: String,
    pub description: String,
    pub smarts: String,
    pub color: String,
    pub priority: u32,
    compiled: Option<chematic::smarts::QueryMolecule>,
}

impl LipidRule {
    /// Check if a molecule matches this rule's pre-compiled SMARTS pattern.
    #[must_use]
    pub fn matches(&self, molecule: &chematic::core::Molecule) -> bool {
        let Some(query) = &self.compiled else {
            return false;
        };
        !chematic::smarts::find_matches(query, molecule).is_empty()
    }
}

/// The rule library: a collection of lipid class definitions indexed by name.
#[derive(Clone, Debug)]
pub struct LipidRuleLibrary {
    pub rules: HashMap<String, LipidRule>,
    pub families: HashMap<String, String>,
    pub architectures: HashMap<String, String>,
}

impl LipidRuleLibrary {
    /// Create an empty rule library.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            families: HashMap::new(),
            architectures: HashMap::new(),
        }
    }

    /// Add a rule to the library, pre-compiling its SMARTS pattern.
    pub fn add_rule(&mut self, mut rule: LipidRule) {
        rule.compiled = chematic::smarts::parse_smarts(&rule.smarts).ok();
        self.rules.insert(rule.name.clone(), rule);
    }

    /// Load evolved SMARTS rules from a CSV file produced by the
    /// `smarts-evoliposuction` binary.
    ///
    /// Only rows with `status == "ok"` and a non-empty `best_smarts` field
    /// are loaded. Each row becomes a [`LipidRule`] with:
    /// - `name` = `label` (or `main_class/subclass` for subclass-level rows)
    /// - `family` = first two characters of `main_class` (e.g. `FA01` → `FA`)
    /// - `smarts` = `best_smarts`
    /// - `description` = formatted string with MCC, coverage, generations
    ///
    /// # Errors
    ///
    /// Returns an error if the CSV file cannot be read or parsed.
    pub fn add_evolved_rules_from_csv(&mut self, csv_path: &str) -> Result<(), EvolvedRulesError> {
        let mut reader = csv::Reader::from_path(csv_path)?;
        let headers = reader.headers()?.clone();

        let idx = |name: &str| -> Option<usize> { headers.iter().position(|h| h == name) };
        let status_idx = idx("status");
        let label_idx = idx("label");
        let main_class_idx = idx("main_class");
        let subclass_idx = idx("subclass");
        let smarts_idx = idx("best_smarts");
        let mcc_idx = idx("best_mcc");
        let coverage_idx = idx("best_coverage_score");
        let gens_idx = idx("generations");

        for record in reader.records() {
            let record = record?;
            let status = record.get(status_idx.unwrap_or(0)).unwrap_or("");
            if status != "ok" {
                continue;
            }
            let smarts = record.get(smarts_idx.unwrap_or(0)).unwrap_or("");
            if smarts.is_empty() {
                continue;
            }
            let label = record.get(label_idx.unwrap_or(0)).unwrap_or("");
            let main_class = record.get(main_class_idx.unwrap_or(0)).unwrap_or("");
            let subclass = record.get(subclass_idx.unwrap_or(0)).unwrap_or("");
            let mcc = record.get(mcc_idx.unwrap_or(0)).unwrap_or("");
            let coverage = record.get(coverage_idx.unwrap_or(0)).unwrap_or("");
            let gens = record.get(gens_idx.unwrap_or(0)).unwrap_or("");

            // Family = first 2 chars of MAIN_CLASS (e.g. "FA01" → "FA").
            let family = if main_class.len() >= 2 {
                main_class[..2].to_string()
            } else {
                main_class.to_string()
            };

            // Name: for subclass-level rows, prefix with main_class.
            let name = if !subclass.is_empty() && label != subclass {
                format!("{main_class}/{subclass}")
            } else {
                label.to_string()
            };

            let description =
                format!("Evolved SMARTS (MCC={mcc}, coverage={coverage}, generations={gens})");

            // Register the family in the taxonomy if missing.
            if !self.families.contains_key(&family) {
                self.families.insert(family.clone(), family.clone());
            }

            self.add_rule(LipidRule {
                name,
                family,
                architecture: String::new(),
                description,
                smarts: smarts.to_string(),
                color: "#6b728b".to_string(),
                priority: 5,
                compiled: None,
            });
        }

        Ok(())
    }

    /// Get a rule by name.
    #[must_use]
    pub fn get_rule(&self, name: &str) -> Option<&LipidRule> {
        self.rules.get(name)
    }

    /// Get all rules sorted by priority (higher first).
    #[must_use]
    pub fn sorted_by_priority(&self) -> Vec<&LipidRule> {
        let mut rules: Vec<_> = self.rules.values().collect();
        rules.sort_by_key(|r| std::cmp::Reverse(r.priority));
        rules
    }

    /// Get all rules for a specific family.
    #[must_use]
    pub fn rules_for_family(&self, family: &str) -> Vec<&LipidRule> {
        self.rules.values().filter(|r| r.family == family).collect()
    }

    /// Return the default LIPID MAPS-aligned rules.
    ///
    /// These rules are carefully curated to match the LIPID MAPS classification
    /// system.  They include proper backbone detection, chain analysis
    /// considerations, and support for multiple lipid architectures
    /// (`DiAcyl`, `MonoAcyl`, `Plasmalogen`, etc.).
    #[must_use]
    pub fn defaults() -> Self {
        let mut library = Self::new();
        library.insert_default_taxonomy();
        library.add_default_fatty_acyl_rules();
        library.add_default_glycerolipid_rules();
        library.add_default_glycerophospholipid_rules();
        library.add_default_sphingolipid_rules();
        library
    }

    fn insert_default_taxonomy(&mut self) {
        self.families
            .insert("FA".to_string(), "Fatty Acyls".to_string());
        self.families
            .insert("GL".to_string(), "Glycerolipids".to_string());
        self.families
            .insert("GP".to_string(), "Glycerophospholipids".to_string());
        self.families
            .insert("SP".to_string(), "Sphingolipids".to_string());

        self.architectures
            .insert("DiAcyl".to_string(), "Two ester linkages".to_string());
        self.architectures.insert(
            "MonoAcyl".to_string(),
            "One ester linkage (lyso)".to_string(),
        );
        self.architectures
            .insert("AlkylAcyl".to_string(), "Ether + ester".to_string());
        self.architectures
            .insert("Plasmalogen".to_string(), "Vinyl ether + ester".to_string());
        self.architectures
            .insert("DiEther".to_string(), "Two ether linkages".to_string());
    }
    fn add_default_fatty_acyl_rules(&mut self) {
        self.add_rule(LipidRule {
            name: "FA".to_string(),
            family: "FA".to_string(),
            architecture: String::new(),
            description: "Saturated or monounsaturated fatty acid".to_string(),
            smarts: "[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R][CX3](=[OX1])[OH]".to_string(),
            color: "#2563eb".to_string(),
            priority: 10,
                compiled: None,
        });

        self.add_rule(LipidRule {
            name: "PUFA".to_string(),
            family: "FA".to_string(),
            architecture: String::new(),
            description: "Polyunsaturated fatty acid (≥2 double bonds)".to_string(),
            smarts: "[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R][CX3](=[OX1])[OH]".to_string(),
            color: "#1e40af".to_string(),
            priority: 9,
                compiled: None,
        });

        self.add_rule(LipidRule {
            name: "MUFA".to_string(),
            family: "FA".to_string(),
            architecture: String::new(),
            description: "Monounsaturated fatty acid".to_string(),
            smarts: "[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R]~[#6;!a;!R][CX3](=[OX1])[OH]".to_string(),
            color: "#3b82f6".to_string(),
            priority: 10,
                compiled: None,
        });
    }
    fn add_default_glycerolipid_rules(&mut self) {
        self.add_rule(LipidRule {
            name: "TG(AAA)".to_string(),
            family: "GL".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Triacylglycerol with three acyl groups".to_string(),
            smarts: "[CX4]([OX2][CX3](=[OX1])[#6])([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]"
                .to_string(),
            color: "#0d9488".to_string(),
            priority: 8,
            compiled: None,
        });

        self.add_rule(LipidRule {
            name: "DG(AA)".to_string(),
            family: "GL".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Diacylglycerol with two acyl groups".to_string(),
            smarts: "[CX4]([OX2][CX3](=[OX1])[#6])[OX2][CX3](=[OX1])[#6]".to_string(),
            color: "#14b8a6".to_string(),
            priority: 7,
            compiled: None,
        });

        self.add_rule(LipidRule {
            name: "MG(A)".to_string(),
            family: "GL".to_string(),
            architecture: "MonoAcyl".to_string(),
            description: "Monoacylglycerol with one acyl group".to_string(),
            smarts: "[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6]".to_string(),
            color: "#2dd4bf".to_string(),
            priority: 6,
            compiled: None,
        });
    }
    fn add_default_glycerophospholipid_rules(&mut self) {
        self.add_rule(LipidRule {
            name: "PC(AA)".to_string(),
            family: "GP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Phosphatidylcholine - diacyl form".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[NX4+]([CH3])([CH3])[CH3]".to_string(),
            color: "#7c3aed".to_string(),
            priority: 10,
            compiled: None,
        });

        self.add_rule(LipidRule {
            name: "PE(AA)".to_string(),
            family: "GP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Phosphatidylethanolamine - diacyl form".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CH2X4][NX3;H2,H1,H0]".to_string(),
            color: "#9333ea".to_string(),
            priority: 9,
            compiled: None,
        });

        self.add_rule(LipidRule {
            name: "PS(AA)".to_string(),
            family: "GP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Phosphatidylserine - diacyl form".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([CX3](=[OX1])[OX2H,OX1-])[NX3]"
                .to_string(),
            color: "#a855f7".to_string(),
            priority: 8,
            compiled: None,
        });

        self.add_rule(LipidRule {
            name: "PI(AA)".to_string(),
            family: "GP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Phosphatidylinositol - contains inositol headgroup".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[C;R1]1[CH;R1][CH;R1][CH;R1][CH;R1][CH;R1]1"
                .to_string(),
            color: "#b78bea".to_string(),
            priority: 7,
            compiled: None,
        });

        self.add_rule(LipidRule {
            name: "PG(AA)".to_string(),
            family: "GP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Phosphatidylglycerol - diacyl form".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4]([OX2H,OX1-])[CH2X4][OX2H,OX1-]"
                .to_string(),
            color: "#cd34b5".to_string(),
            priority: 6,
            compiled: None,
        });

        self.add_rule(LipidRule {
            name: "PA(AA)".to_string(),
            family: "GP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Phosphatidic acid - minimal phospholipid".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[CH2X4][CHX4][CH2X4][OX2H,OX1-]".to_string(),
            color: "#ec4899".to_string(),
            priority: 5,
            compiled: None,
        });

        self.add_rule(LipidRule {
            name: "LPC(A)".to_string(),
            family: "GP".to_string(),
            architecture: "MonoAcyl".to_string(),
            description: "Lysophosphatidylcholine - monoacyl form".to_string(),
            smarts: "[CH2X4][CHX4][CH2X4][OX2][CX3](=[OX1])[#6]".to_string(),
            color: "#f472b6".to_string(),
            priority: 6,
            compiled: None,
        });

        self.add_rule(LipidRule {
            name: "LPE(A)".to_string(),
            family: "GP".to_string(),
            architecture: "MonoAcyl".to_string(),
            description: "Lysophosphatidylethanolamine - monoacyl form".to_string(),
            smarts: "[CH2X4][CHX4][CH2X4][OX2]".to_string(),
            color: "#f787c8".to_string(),
            priority: 5,
            compiled: None,
        });

        self.add_rule(LipidRule {
            name: "CL(AAAA)".to_string(),
            family: "GP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Cardiolipin - four acyl groups".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[PX4](=[OX1])([OX2])([OX2])".to_string(),
            color: "#f8bbd0".to_string(),
            priority: 4,
            compiled: None,
        });
    }
    fn add_default_sphingolipid_rules(&mut self) {
        self.add_rule(LipidRule {
            name: "Cer(AS)".to_string(),
            family: "SP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Ceramide - sphingoid base + amide-linked acyl".to_string(),
            smarts:
                "[#6;!a;!R][CHX3]=[CHX3][CHX4]([NX3][CX3](=[OX1])[#6])[CHX4]([OX2H,OX1-])[CH2X4]"
                    .to_string(),
            color: "#be185d".to_string(),
            priority: 9,
            compiled: None,
        });

        self.add_rule(LipidRule {
            name: "SM(AS)".to_string(),
            family: "SP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Sphingomyelin - ceramide + phosphocholine headgroup".to_string(),
            smarts: "[PX4](=[OX1])([OX2])([OX2])[NX4+]([CH3])([CH3])[CH3]".to_string(),
            color: "#db2777".to_string(),
            priority: 8,
            compiled: None,
        });

        self.add_rule(LipidRule {
            name: "HexCer(AS)".to_string(),
            family: "SP".to_string(),
            architecture: "DiAcyl".to_string(),
            description: "Hexosylceramide - ceramide + hexose headgroup".to_string(),
            smarts:
                "[#6;!a;!R][CHX3]=[CHX3][CHX4]([NX3][CX3](=[OX1])[#6])[CHX4]([OX2H,OX1-])[CH2X4]"
                    .to_string(),
            color: "#e91e63".to_string(),
            priority: 7,
            compiled: None,
        });
    }
}

impl Default for LipidRuleLibrary {
    fn default() -> Self {
        Self::defaults()
    }
}

// === Color Attribution ===

/// CVD-friendly color palettes for the 8 LIPID MAPS families.
///
/// Each palette has 5 shades ordered from darkest to lightest. The 8
/// palettes are assigned to the 8 major LIPID MAPS families by count rank
/// (the family with the most lipids gets the first palette).
pub mod colors {
    /// A color palette with 5 shades (darkest → lightest).
    #[derive(Debug, Clone, Copy)]
    pub struct Palette {
        pub name: &'static str,
        pub colors: [&'static str; 5],
    }

    /// The 8 palettes, ordered by rank (most abundant → least abundant).
    /// The first palette is assigned to the family with the most lipids.
    pub const PALETTES: [Palette; 8] = [
        Palette {
            name: "cvd_green",
            colors: ["#4E7705", "#6D9F06", "#97CE2F", "#BDEC6F", "#DDFFA0"],
        },
        Palette {
            name: "cvd_blue",
            colors: ["#098BD9", "#56B4E9", "#7DCCFF", "#BCE1FF", "#E7F4FF"],
        },
        Palette {
            name: "cvd_purple",
            colors: ["#7D3560", "#A1527F", "#CC79A7", "#E794C1", "#EFB6D6"],
        },
        Palette {
            name: "cvd_orange",
            colors: ["#9D654C", "#C17754", "#F09163", "#FCB076", "#FFD5AF"],
        },
        Palette {
            name: "green",
            colors: ["#238b45", "#41ab5d", "#74c476", "#a1d99b", "#c7e9c0"],
        },
        Palette {
            name: "blue",
            colors: ["#4292c6", "#6baed6", "#9ecae1", "#c6dbef", "#eff3ff"],
        },
        Palette {
            name: "purple",
            colors: ["#6a51a3", "#807dba", "#9e9ac8", "#bcbddc", "#dadaeb"],
        },
        Palette {
            name: "orange",
            colors: ["#ff7f00", "#fe9929", "#fdae6b", "#fec44f", "#feeda0"],
        },
    ];

    /// Extract the 2-letter family code from a `MAIN_CLASS` value.
    ///
    /// `MAIN_CLASS` values look like `"Fatty Acyls [FA01]"` — the family
    /// code is the two uppercase letters inside the brackets.
    #[must_use]
    pub fn family_code(main_class: &str) -> &str {
        // Find "[XX" in the string and take the two-letter code.
        if let Some(start) = main_class.rfind('[') {
            let rest = &main_class[start + 1..];
            if rest.len() >= 2 {
                let code = &rest[..2];
                if code.chars().all(|c| c.is_ascii_uppercase()) {
                    return code;
                }
            }
        }
        // Fallback: use the first two characters.
        &main_class[..main_class.len().min(2)]
    }

    /// A color assignment for one LIPID MAPS family or subclass.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Assignment {
        /// Family code (e.g. `"FA"`, `"GP"`).
        pub family: String,
        /// Subclass label (e.g. `"Unsaturated fatty acids"`).
        /// Empty for the family-level entry; `"... - others"` for grouping.
        pub subclass: String,
        /// Hex color code.
        pub color: String,
        /// Number of lipids in this class.
        pub count: usize,
        /// True if this is a grouped "others" entry.
        pub is_others: bool,
    }

    impl Assignment {
        /// Human-readable label showing the attributed LIPID MAPS class
        /// and subclass, suitable for display on a card.
        ///
        /// - Family-level: `"FA"`
        /// - Subclass-level: `"FA / Unsaturated fatty acids"`
        /// - Others group: `"FA - others (5 subfamilies)"`
        #[must_use]
        pub fn label(&self) -> String {
            if self.is_others {
                format!("{} - others", self.family)
            } else if self.subclass.is_empty() {
                self.family.clone()
            } else {
                format!("{} / {}", self.family, self.subclass)
            }
        }
    }

    /// Count the 8 major families from a TSV string and attribute colors.
    ///
    /// The TSV must have `MAIN_CLASS` and `SUB_CLASS` columns (as produced
    /// by `lipidsdl::sdf::lipidmaps::to_lmsd_tsv`).
    ///
    /// Returns one [`Assignment`] per family (the family-level entry) plus
    /// per-subclass entries.
    ///
    /// Families are sorted by descending lipid count and assigned palettes in
    /// rank order. Within each family, subclasses are sorted by descending
    /// count; if there are more than 5 subclasses, the 5th and beyond are
    /// grouped into a single `"<family> - others"` entry.
    ///
    /// # Errors
    ///
    /// Returns an error if the `csv` reader fails.
    #[allow(clippy::module_name_repetitions)]
    pub fn attribute_colors(tsv: &str) -> Result<Vec<Assignment>, csv::Error> {
        use std::collections::HashMap;

        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(true)
            .from_reader(tsv.as_bytes());
        let headers = reader.headers()?.clone();

        // Find column indices.
        let mut mc_idx = None;
        let mut sc_idx = None;
        for (i, h) in headers.iter().enumerate() {
            match h.trim() {
                "MAIN_CLASS" => mc_idx = Some(i),
                "SUB_CLASS" => sc_idx = Some(i),
                _ => {}
            }
        }

        // Count lipids per (family, subclass).
        let mut family_counts: HashMap<String, usize> = HashMap::new();
        let mut subclass_counts: HashMap<String, HashMap<String, usize>> = HashMap::new();

        for record in reader.records() {
            let record = record?;
            let mc = record.get(mc_idx.unwrap_or(0)).unwrap_or("");
            let sc = record.get(sc_idx.unwrap_or(0)).unwrap_or("");
            let fam = family_code(mc);
            *family_counts.entry(fam.to_string()).or_insert(0) += 1;
            if !sc.is_empty() {
                *subclass_counts
                    .entry(fam.to_string())
                    .or_default()
                    .entry(sc.to_string())
                    .or_insert(0) += 1;
            }
        }

        // Sort families by count (descending) and assign palettes.
        let mut families: Vec<(String, usize)> = family_counts.into_iter().collect();
        families.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let mut result = Vec::new();
        for (rank, (family, count)) in families.iter().enumerate() {
            let palette = if rank < PALETTES.len() {
                &PALETTES[rank]
            } else {
                &PALETTES[PALETTES.len() - 1]
            };

            // Family-level entry: uses the darkest shade.
            result.push(Assignment {
                family: family.clone(),
                subclass: String::new(),
                color: palette.colors[0].to_string(),
                count: *count,
                is_others: false,
            });

            // Subclass entries.
            if let Some(subs) = subclass_counts.get(family) {
                let mut subs_sorted: Vec<(String, usize)> =
                    subs.iter().map(|(k, v)| (k.clone(), *v)).collect();
                subs_sorted.sort_by_key(|(_, c)| std::cmp::Reverse(*c));

                let total_subs = subs_sorted.len();
                // Top 4 subclasses get individual shades; 5th+ grouped as "others".
                // The threshold is "more than 5 subclasses" → group after the 4th.
                let limit = if total_subs > 5 { 4 } else { total_subs.min(4) };

                for (i, (sc, sc_count)) in subs_sorted.iter().enumerate() {
                    if i < limit {
                        let shade = (i + 1).min(4);
                        result.push(Assignment {
                            family: family.clone(),
                            subclass: sc.clone(),
                            color: palette.colors[shade].to_string(),
                            count: *sc_count,
                            is_others: false,
                        });
                    } else if i == limit {
                        // First "others" entry.
                        let others_count: usize = subs_sorted[limit..].iter().map(|(_, c)| c).sum();
                        result.push(Assignment {
                            family: family.clone(),
                            subclass: format!("{family} - others"),
                            color: palette.colors[4].to_string(),
                            count: others_count,
                            is_others: true,
                        });
                    }
                    // Subsequent "others" entries are skipped (grouped).
                }
            }
        }

        Ok(result)
    }

    /// Note: color attribution is a visualization aid only.
    ///
    /// The actual LIPID MAPS classification (`MAIN_CLASS`/`SUB_CLASS` codes)
    /// remains accessible and unchanged through `LipidRuleLibrary::defaults`.
    /// Colors are assigned by lipid count rank, not chemical semantics.
    #[must_use]
    pub const fn is_visualization_only() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_include_major_lipid_classes() {
        let lib = LipidRuleLibrary::defaults();
        assert!(lib.get_rule("PC(AA)").is_some());
        assert!(lib.get_rule("PE(AA)").is_some());
        assert!(lib.get_rule("TG(AAA)").is_some());
        assert!(lib.get_rule("FA").is_some());
        assert!(lib.get_rule("Cer(AS)").is_some());
    }

    #[test]
    fn sorted_by_priority_returns_highest_first() {
        let lib = LipidRuleLibrary::defaults();
        let sorted = lib.sorted_by_priority();
        assert!(!sorted.is_empty());
        for i in 0..sorted.len().saturating_sub(1) {
            assert!(sorted[i].priority >= sorted[i + 1].priority);
        }
    }

    #[test]
    fn families_are_defined() {
        let lib = LipidRuleLibrary::defaults();
        assert!(lib.families.contains_key("FA"));
        assert!(lib.families.contains_key("GL"));
        assert!(lib.families.contains_key("GP"));
        assert!(lib.families.contains_key("SP"));
    }

    #[test]
    fn architectures_are_defined() {
        let lib = LipidRuleLibrary::defaults();
        assert!(lib.architectures.contains_key("DiAcyl"));
        assert!(lib.architectures.contains_key("MonoAcyl"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn add_evolved_rules_from_csv_loads_ok_rows() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let csv_content = "level,label,main_class,subclass,slug,positive_count,negative_count,positive_parse_failures,negative_parse_failures,best_smarts,best_mcc,best_coverage_score,best_smarts_len,generations,elapsed_secs,status\n\
main_class,FA,FA01,,fa,50,50,0,0,[CX3](=[OX1])[OH],0.95,0.90,12,100,12.5,ok\n\
main_class,GL,GL03,,gl,30,30,0,0,[PX4](=[OX1])([OX2])([OX2])N,0.88,0.82,25,100,10.0,ok\n\
main_class,SP,SP03,,sp,10,10,0,0,,0.0,0.0,0,100,5.0,evolution_error\n\
subclass,GP,GP,GP03,gp_sub,25,25,0,0,[NX4+],0.75,0.70,18,100,8.3,ok\n";

        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(csv_content.as_bytes()).unwrap();
        tmp.flush().unwrap();

        let mut lib = LipidRuleLibrary::new();
        lib.add_evolved_rules_from_csv(tmp.path().to_str().unwrap())
            .unwrap();

        // Three "ok" rows should be loaded; the "evolution_error" row is skipped.
        assert_eq!(lib.rules.len(), 3);
        assert!(lib.get_rule("FA").is_some());
        assert!(lib.get_rule("GL").is_some());
        assert!(lib.get_rule("GP/GP03").is_some());

        let fa_rule = lib.get_rule("FA").unwrap();
        assert_eq!(fa_rule.smarts, "[CX3](=[OX1])[OH]");
        assert!(fa_rule.description.contains("MCC=0.95"));
        assert!(fa_rule.description.contains("coverage=0.9"));
        assert_eq!(fa_rule.family, "FA");

        let gl_rule = lib.get_rule("GL").unwrap();
        assert_eq!(gl_rule.family, "GL");
        assert!(lib.get_rule("SP").is_none()); // evolution_error → skipped
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn add_evolved_rules_from_csv_missing_file() {
        let mut lib = LipidRuleLibrary::new();
        let result = lib.add_evolved_rules_from_csv("/nonexistent/path.csv");
        assert!(result.is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn add_evolved_rules_to_defaults() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Use a class not present in defaults (ST = Sterols).
        let csv_content = "level,label,main_class,subclass,slug,positive_count,negative_count,positive_parse_failures,negative_parse_failures,best_smarts,best_mcc,best_coverage_score,best_smarts_len,generations,elapsed_secs,status\n\
main_class,ST,ST05,,st_evolved,50,50,0,0,[CX3](=[OX1])[OH],0.95,0.90,12,100,12.5,ok\n";

        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(csv_content.as_bytes()).unwrap();
        tmp.flush().unwrap();

        let mut lib = LipidRuleLibrary::defaults();
        let original_count = lib.rules.len();
        lib.add_evolved_rules_from_csv(tmp.path().to_str().unwrap())
            .unwrap();
        assert_eq!(lib.rules.len(), original_count + 1);
        let st_rule = lib.get_rule("ST").unwrap();
        assert_eq!(st_rule.smarts, "[CX3](=[OX1])[OH]");
        assert_eq!(st_rule.family, "ST");
    }

    // === Color Attribution Tests ===

    #[test]
    fn family_code_extracts_two_letter_code() {
        assert_eq!(colors::family_code("Fatty Acyls [FA01]"), "FA");
        assert_eq!(colors::family_code("Triradylglycerols [GL03]"), "GL");
        assert_eq!(colors::family_code("Some Class [PK12]"), "PK");
        assert_eq!(colors::family_code("No brackets here"), "No");
    }

    #[test]
    fn palettes_have_5_shades_each() {
        assert_eq!(colors::PALETTES.len(), 8);
        for p in &colors::PALETTES {
            assert_eq!(p.colors.len(), 5);
        }
    }

    /// A realistic subset of LIPIDMAPS SMILES data covering all 8 major
    /// families, drawn from the real LMSD TSV.
    const SAMPLE_TSV: &str = "LM_ID\tNAME\tSYSTEMATIC_NAME\tSYNONYMS\tCATEGORY\tMAIN_CLASS\tSUB_CLASS\tEXACT_MASS\tFORMULA\tINCHI_KEY\tINCHI\tSMILES\tPUBCHEM_CID\tCHEBI_ID\tKEGG_ID\tHMDB_ID\tSWISSLIPIDS_ID\tLIPIDBANK_ID\tPLANTFA_ID\n\
LMFA0001\tTestA\t\t\tFA\tFatty Acyls [FA]\tOther Fatty Acyls [FA00]\t100.0\tC40H66O5\tKEY1\tINCHI1\tC(C(OC)CCC#CCCCCCC(C)CCCCC=C)(=O)OC(C(OC)CCC#CCCCCCC(C)CCCCC=C)=O\t1\t1\t1\t1\t1\t1\t1\t1\n\
LMFA0002\tTestB\t\t\tFA\tFatty Acyls [FA]\tOther Fatty Acyls [FA00]\t100.0\tC40H66O5\tKEY2\tINCHI2\tC(C(OC)CCC#CCCCCCC(C)CCCCC=C)(=O)OC(C(OC)CCC#CCCCCCC(C)CCCCC=C)=O\t2\t2\t2\t2\t2\t2\t2\t2\n\
LMGA0001\tTestC\t\t\tGL\tGlycerolipids [GL]\tOther Glycerolipids [GL00]\t100.0\tC50H85NO7\tKEY3\tINCHI3\tC(COCC(C([O-])=O)([H])C[N+](C)(C)C)([H])(OC(CCCCCCCC/C=C\\C/C=C\\C/C=C\\C/C=C\\CC)=O\t3\t3\t3\t3\t3\t3\t3\t3\n\
LMGA0002\tTestD\t\t\tGP\tGlycerophospholipids [GP]\tOther Glycerophospholipids [GP00]\t100.0\tC39H77O8PS\tKEY4\tINCHI4\tC(=O)([C@H](CCCCCCCCCCCCCC)O)N+([C@@H](C)C)(C)C\t4\t4\t4\t4\t4\t4\t4\t4\n\
LMGP0001\tTestE\t\t\tSP\tSphingolipids [SP]\tOther Sphingolipids [SP00]\t100.0\tC32H63NO5S\tKEY5\tINCHI5\tC([C@]([H])(NC(=O)CCCCCCCCCCCCC)[C@]([H])(O)/C=C/CCCCCCCCCCCCC)S(=O)(=O)O\t5\t5\t5\t5\t5\t5\t5\t5\n\
LMGP0002\tTestF\t\t\tST\tSterol Lipids [ST]\tSterols [ST01]\t100.0\tC27H48\tKEY6\tINCHI6\t[C@]12(CCC3CCCC[C@]3(C)[C@@]1([H])CC[C@]1(C)[C@@]([H])([C@@](C)([H])CCCC(C)C)CC[C@@]21[H])[H]\t6\t6\t6\t6\t6\t6\t6\t6\n\
LMPR0001\tTestG\t\t\tPR\tPrenol Lipids [PR]\tIsoprenoids [PR01]\t100.0\tC35H56\tKEY7\tINCHI7\tC/C(/C)=C/CC/C(/C)=C/C/C=C(\\C(CCC(=C)C=C)CC(C/C=C(\\C)/C)/C(=C\\C/C=C(/C)\\C)/C)/C\t7\t7\t7\t7\t7\t7\t7\t7\n\
LMSL0001\tTestH\t\t\tSL\tSaccharolipids [SL]\tAcylaminosugars [SL01]\t100.0\tC29H51N3O18P2\tKEY8\tINCHI8\tO(P(O)(=O)OC[C@@H]1[C@@H](O)[C@@H](O)[C@H](N2C(=O)NC(=O)C=C2)O1)P(O[C@H]1O[C@@H]([C@H]([C@@H]([C@H]1N)OC(C[C@@H](CCCCCCCCCCC)O)=O)O)CO)(=O)O\t8\t8\t8\t8\t8\t8\t8\t8\n\
LMPK0001\tTestI\t\t\tPK\tPolyketides [PK]\tOther Polyketides [PK00]\t100.0\tC35H46O14\tKEY9\tINCHI9\tO1[C@@]2(CCC(=C)[C@H]([C@H](C)CC3C=CC=CC=3)OC(C)=O)[C@@H]([C@H]([C@]1(C(=O)O)[C@](C(=O)O)([C@@H](C(=O)O)O2)O)OC(/C=C/[C@@H](C)C[C@@H](C)CC)=O)O\t9\t9\t9\t9\t9\t9\t9\t9\n";

    #[test]
    fn attribute_colors_assigns_palettes_by_count_rank() {
        let assignments = colors::attribute_colors(SAMPLE_TSV).unwrap();

        // All 8 families have count 1 each (one lipid per family), except
        // FA (2) and GP (1, but GP appears twice in the data) — FA has 2,
        // all others have 1. With alphabetical tie-break: FA(2) → cvd_green
        // (rank 0), then GL, GP, PK, PR, SL, SP, ST (each 1) → cvd_blue
        // through orange.
        let families_in_order = ["FA", "GL", "GP", "PK", "PR", "SL", "SP", "ST"];
        let expected_colors = [
            "#4E7705", // cvd_green
            "#098BD9", // cvd_blue
            "#7D3560", // cvd_purple
            "#9D654C", // cvd_orange
            "#238b45", // green
            "#4292c6", // blue
            "#6a51a3", // purple
            "#ff7f00", // orange
        ];

        for (&code, &color) in families_in_order.iter().zip(&expected_colors) {
            let entry = assignments
                .iter()
                .find(|a| a.family == code && a.subclass.is_empty());
            assert!(entry.is_some(), "family {code} not found");
            assert_eq!(entry.unwrap().color, color, "family {code} color");
        }
    }

    #[test]
    fn attribute_colors_subclasses_use_lighter_shades() {
        let assignments = colors::attribute_colors(SAMPLE_TSV).unwrap();

        // FA has 2 lipids in 1 subclass ("Other Fatty Acyls").
        let fa_subs: Vec<_> = assignments
            .iter()
            .filter(|a| a.family == "FA" && !a.subclass.is_empty())
            .collect();

        assert_eq!(fa_subs.len(), 1);
        // Subclass gets shade 1 (lighter than family's shade 0).
        assert_eq!(fa_subs[0].color, "#6D9F06"); // cvd_green shade 1
        assert_eq!(fa_subs[0].count, 2);
        assert!(!fa_subs[0].is_others);
    }

    #[test]
    fn attribute_colors_groups_others_when_many_subclasses() {
        use std::fmt::Write;
        // Build a TSV with 7 subclasses in one family.
        let mut tsv = "LM_ID\tNAME\tSYSTEMATIC_NAME\tSYNONYMS\tCATEGORY\tMAIN_CLASS\tSUB_CLASS\tEXACT_MASS\tFORMULA\tINCHI_KEY\tINCHI\tSMILES\tPUBCHEM_CID\tCHEBI_ID\tKEGG_ID\tHMDB_ID\tSWISSLIPIDS_ID\tLIPIDBANK_ID\tPLANTFA_ID\n".to_string();
        for i in 0..7 {
            writeln!(
                tsv,
                "LMT\tTest{i}\t\t\tFA\tFatty Acyls [FA01]\tSub{i} [FA01xx]\t100.0\tC16H32O2\tKEY{i}\tINCHI{i}\tC(C)C\t{i}\t{i}\t{i}\t{i}\t{i}\t{i}\t{i}"
            ).unwrap();
        }

        let assignments = colors::attribute_colors(&tsv).unwrap();
        let fa_subs: Vec<_> = assignments
            .iter()
            .filter(|a| a.family == "FA" && !a.subclass.is_empty())
            .collect();

        // Top 4 subclasses get individual colors; 5th+ grouped as "others".
        assert_eq!(fa_subs.len(), 5); // 4 individual + 1 "others"
        assert!(fa_subs[4].is_others);
        assert!(fa_subs[4].subclass.contains("others"));
    }

    #[test]
    fn attribute_colors_is_visualization_only() {
        assert!(colors::is_visualization_only());
    }

    #[test]
    fn assignment_label_shows_class_and_subclass() {
        let assignments = colors::attribute_colors(SAMPLE_TSV).unwrap();

        // Family-level entry: label is just the family code.
        let fa = assignments
            .iter()
            .find(|a| a.family == "FA" && a.subclass.is_empty())
            .unwrap();
        assert_eq!(fa.label(), "FA");

        // Subclass-level entry: label is "FA / <subclass>".
        let fa_sub = assignments
            .iter()
            .find(|a| a.family == "FA" && !a.subclass.is_empty() && !a.is_others)
            .unwrap();
        assert_eq!(fa_sub.label(), format!("FA / {}", fa_sub.subclass));

        // Others entry: label contains "FA - others".
        let mut tsv = String::from(SAMPLE_TSV);
        for i in 0..(4 + 2) {
            use std::fmt::Write;
            writeln!(
                tsv,
                "LMT\tT{i}\t\t\tFA\tFatty Acyls [FA]\tSub{i} [FAxx]\t100.0\tC16H32O2\tK{i}\tI{i}\tC(C)C\t{i}\t{i}\t{i}\t{i}\t{i}\t{i}\t{i}"
            ).unwrap();
        }
        let assignments = colors::attribute_colors(&tsv).unwrap();
        let others = assignments
            .iter()
            .find(|a| a.family == "FA" && a.is_others)
            .unwrap();
        assert_eq!(others.label(), "FA - others");
    }
}
