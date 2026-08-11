//! Fatty acyl chain analysis for lipid characterization.
//!
//! This module identifies and characterizes individual fatty acyl/radyl groups
//! within lipid molecules, computing chain properties like:
//! - Carbon count
//! - Number of C=C unsaturations
//! - Position of unsaturations
//! - Stereochemistry
//! - Modifications (OH, OOH, epoxides)

use chematic::core::Molecule;

/// Properties of a single fatty acyl or radyl chain.
#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct AcylChain {
    /// Total number of carbons in the chain
    pub carbon_count: u32,
    /// Number of C=C double bonds
    pub double_bonds: u32,
    /// Number of C≡C triple bonds
    pub triple_bonds: u32,
    /// Positions of double bonds (0-indexed from carboxyl)
    pub double_bond_positions: Vec<u32>,
    /// Does this chain have a hydroxyl group?
    pub has_hydroxyl: bool,
    /// Does this chain have a hydroperoxide?
    pub has_hydroperoxide: bool,
    /// Does this chain have an epoxide?
    pub has_epoxide: bool,
    /// Is this chain part of an ether linkage (not ester)?
    pub is_ether_linked: bool,
    /// Is this a plasmalogen (1Z-alkenyl ether)?
    pub is_plasmalogen: bool,
}

impl AcylChain {
    /// Classify this chain by unsaturation degree.
    #[must_use]
    pub const fn unsaturation_class(&self) -> ChainType {
        match self.double_bonds {
            0 => ChainType::Saturated,
            1 => ChainType::MUFA,
            n if n >= 2 => ChainType::PUFA,
            _ => ChainType::Saturated,
        }
    }
}

/// Classification of a chain by unsaturation degree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChainType {
    /// No C=C bonds (saturated)
    Saturated,
    /// Exactly 1 C=C bond (monounsaturated)
    MUFA,
    /// 2+ C=C bonds (polyunsaturated)
    PUFA,
}

/// Composition of a lipid based on its acyl chains.
#[derive(Clone, Debug)]
pub struct ChainComposition {
    /// All detected chains
    pub chains: Vec<AcylChain>,
    /// Summary of chain counts by type
    pub saturated_count: usize,
    pub mufa_count: usize,
    pub pufa_count: usize,
}

impl ChainComposition {
    /// Create from a set of chains.
    #[must_use]
    pub fn from_chains(chains: Vec<AcylChain>) -> Self {
        let saturated_count = chains.iter().filter(|c| c.double_bonds == 0).count();
        let mufa_count = chains.iter().filter(|c| c.double_bonds == 1).count();
        let pufa_count = chains.iter().filter(|c| c.double_bonds >= 2).count();

        Self {
            chains,
            saturated_count,
            mufa_count,
            pufa_count,
        }
    }

    /// Lipid nomenclature string: chain summaries.
    /// E.g., "16:0/18:1" for a 16:0 and 18:1 acyl chain.
    #[must_use]
    pub fn nomenclature(&self) -> String {
        self.chains
            .iter()
            .map(|c| format!("{}:{}", c.carbon_count, c.double_bonds))
            .collect::<Vec<_>>()
            .join("/")
    }
}

/// Analyze the fatty acyl chains in a molecule.
///
/// This is a stub implementation. Production code would:
/// 1. Identify acyl/radyl attachment points (esters, ethers, amides)
/// 2. Traverse from each attachment along the hydrocarbon chain
/// 3. Count and position unsaturations
/// 4. Detect stereochemistry (E/Z, cis/trans)
/// 5. Identify modifications
#[must_use]
pub const fn analyze_chains(_molecule: &Molecule) -> Option<ChainComposition> {
    // TODO: Implement chain traversal and analysis
    // For now, return None to indicate chains not yet analyzed
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_type_classification() {
        let saturated = AcylChain {
            carbon_count: 16,
            double_bonds: 0,
            triple_bonds: 0,
            double_bond_positions: vec![],
            has_hydroxyl: false,
            has_hydroperoxide: false,
            has_epoxide: false,
            is_ether_linked: false,
            is_plasmalogen: false,
        };

        assert_eq!(saturated.unsaturation_class(), ChainType::Saturated);
    }

    #[test]
    fn composition_counts_chains() {
        let chains = vec![
            AcylChain {
                carbon_count: 16,
                double_bonds: 0,
                triple_bonds: 0,
                double_bond_positions: vec![],
                has_hydroxyl: false,
                has_hydroperoxide: false,
                has_epoxide: false,
                is_ether_linked: false,
                is_plasmalogen: false,
            },
            AcylChain {
                carbon_count: 18,
                double_bonds: 1,
                triple_bonds: 0,
                double_bond_positions: vec![9],
                has_hydroxyl: false,
                has_hydroperoxide: false,
                has_epoxide: false,
                is_ether_linked: false,
                is_plasmalogen: false,
            },
        ];

        let comp = ChainComposition::from_chains(chains);
        assert_eq!(comp.saturated_count, 1);
        assert_eq!(comp.mufa_count, 1);
        assert_eq!(comp.pufa_count, 0);
        assert_eq!(comp.nomenclature(), "16:0/18:1");
    }
}
