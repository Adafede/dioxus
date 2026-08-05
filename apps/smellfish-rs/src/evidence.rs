use crate::model::{MoleculeRow, RdkitDescriptors};

#[derive(Clone, Debug)]
pub struct EvidenceAssessment {
    pub np_likeness: f64,
    pub np_label: String,
    pub ring_family: String,
    pub evidence_notes: Vec<String>,
    pub motif_context: String,
}

pub fn assess_np_evidence(
    descriptors: &RdkitDescriptors,
    motifs: &[String],
    stereo_tags: &[String],
) -> EvidenceAssessment {
    let ring_family = classify_ring_family(descriptors, motifs);
    let mut score = 0.0;
    let mut notes = Vec::new();
    let mut scaffold_hits = 0usize;
    let mut decoration_hits = 0usize;

    if let Some(csp3) = descriptors.fraction_csp3 {
        let contribution = ((csp3 - 0.28) * 4.0).clamp(-1.25, 1.4);
        score += contribution;
        if contribution > 0.35 {
            notes.push(format!("sp3-rich core (fractionCSP3 {csp3:.2})"));
        }
    }

    if let Some(rings) = descriptors.ring_count {
        let contribution = (rings.min(6.0) * 0.18) - 0.15;
        score += contribution;
        if rings >= 2.0 {
            notes.push(format!("ring-rich scaffold ({rings:.0} rings)"));
        }
    }

    if let Some(aromatic) = descriptors.aromatic_ring_count {
        let penalty = (aromatic * 0.35).min(1.25);
        score -= penalty;
        if aromatic >= 2.0 {
            notes.push(format!(
                "aromatic fraction still high ({aromatic:.0} aromatic rings)"
            ));
        }
    }

    if let Some(aliphatic) = descriptors.aliphatic_ring_count {
        let bonus = (aliphatic * 0.28).min(0.84);
        score += bonus;
        if aliphatic >= 1.0 {
            notes.push(format!(
                "aliphatic ring system ({aliphatic:.0} aliphatic rings)"
            ));
        }
    }

    if let Some(tpsa) = descriptors.tpsa {
        if (25.0..=180.0).contains(&tpsa) {
            score += 0.35;
            notes.push(format!("polar enough for NP space (TPSA {tpsa:.1})"));
        } else {
            score -= 0.2;
        }
    }

    if let Some(clogp) = descriptors.clogp {
        if (0.5..=5.5).contains(&clogp) {
            score += 0.25;
        } else if clogp > 6.0 {
            score -= 0.35;
        }
    }

    if let Some(hetero_atoms) = descriptors.hetero_atoms {
        if hetero_atoms >= 2.0 {
            score += 0.3;
            notes.push(format!(
                "heteroatom-rich core ({hetero_atoms:.0} hetero atoms)"
            ));
        } else {
            score -= 0.15;
        }
    }

    if let Some(rotatable_bonds) = descriptors.rotatable_bonds {
        if (1.0..=12.0).contains(&rotatable_bonds) {
            score += 0.15;
        } else if rotatable_bonds > 12.0 {
            score -= 0.25;
        }
    }

    if let Some(hba) = descriptors.hba {
        if hba >= 2.0 {
            score += 0.1;
        }
    }
    if let Some(hbd) = descriptors.hbd {
        if hbd >= 1.0 {
            score += 0.1;
        }
    }

    let stereo_count = stereo_tags.len() as f64;
    if stereo_count > 0.0 {
        let contribution = (stereo_count.min(4.0)) * 0.22;
        score += contribution;
        notes.push(format!(
            "stereochemical richness ({} stereo tags)",
            stereo_tags.len()
        ));
    } else if descriptors.fraction_csp3.unwrap_or(0.0) > 0.35 {
        score -= 0.2;
    }

    match ring_family.as_str() {
        "steroid-like fused ring system"
        | "sugar-like oxygenated ring system"
        | "macrolide-like oxygenated macrocycle"
        | "fused heteroaromatic scaffold"
        | "natural-product-like polycyclic scaffold" => {
            score += 0.8;
            notes.push(format!("{ring_family}"));
        }
        "polyaromatic scaffold" => {
            score -= 0.85;
            notes.push("polyaromatic bias".to_string());
        }
        _ => {}
    }

    for motif in motifs {
        let motif_lower = motif.to_ascii_lowercase();
        if motif_lower.contains("steroid")
            || motif_lower.contains("sugar")
            || motif_lower.contains("macrocycle")
            || motif_lower.contains("lactone")
            || motif_lower.contains("lactam")
            || motif_lower.contains("indole")
            || motif_lower.contains("quinoline")
            || motif_lower.contains("isoquinoline")
            || motif_lower.contains("benzofuran")
            || motif_lower.contains("benzothiophene")
            || motif_lower.contains("quinoxaline")
            || motif_lower.contains("purine")
            || motif_lower.contains("chromone")
            || motif_lower.contains("coumarin")
        {
            scaffold_hits += 1;
        } else {
            decoration_hits += 1;
        }

        if motif_lower.contains("steroid") || motif_lower.contains("sugar") {
            score += 0.35;
        }
        if motif_lower.contains("indole")
            || motif_lower.contains("isoquinoline")
            || motif_lower.contains("quinoline")
            || motif_lower.contains("chromone")
            || motif_lower.contains("coumarin")
            || motif_lower.contains("benzofuran")
            || motif_lower.contains("benzothiophene")
        {
            score += 0.2;
        }
    }

    score = score.clamp(-5.0, 5.0);
    let np_label = np_likeness_label(score).to_string();
    let motif_context = if scaffold_hits > decoration_hits && scaffold_hits > 0 {
        format!("scaffold-heavy motif set ({scaffold_hits} scaffold hits)")
    } else if decoration_hits > scaffold_hits && decoration_hits > 0 {
        format!("decoration-heavy motif set ({decoration_hits} decoration hits)")
    } else if scaffold_hits > 0 || decoration_hits > 0 {
        "balanced motif set".to_string()
    } else {
        "no motif signal".to_string()
    };

    EvidenceAssessment {
        np_likeness: score,
        np_label,
        ring_family,
        evidence_notes: notes,
        motif_context,
    }
}

pub fn verdict_for_row(row: &MoleculeRow) -> String {
    if let Some(err) = row.error.as_deref() {
        return format!("⚠ {err}");
    }

    let has_lotus = !row.lotus_taxa.is_empty();
    let has_pubchem = !row.pubchem_cids.is_empty();
    let score = row.np_likeness;

    if has_lotus && has_pubchem && score >= 0.8 {
        return "Looks legitimate — LOTUS, PubChem, and NP evidence agree.".to_string();
    }
    if has_lotus && score >= 1.5 {
        return "🌿 Strong natural-product evidence from LOTUS and descriptors.".to_string();
    }
    if has_lotus {
        return "🌿 LOTUS-backed natural-product evidence.".to_string();
    }
    if has_pubchem {
        return "📚 PubChem hit only — weak evidence.".to_string();
    }
    if score >= 2.0 {
        return "NP-like scaffold, but database support is still thin.".to_string();
    }
    if score <= -1.0 {
        return "👃 Smells fishy. Citation needed.".to_string();
    }
    "🤨 Citation needed.".to_string()
}

pub fn np_likeness_label(score: f64) -> &'static str {
    if score >= 2.0 {
        "strong NP-like"
    } else if score >= 0.8 {
        "NP-leaning"
    } else if score >= -0.7 {
        "mixed"
    } else {
        "synthetic-leaning"
    }
}

pub fn classify_ring_family(descriptors: &RdkitDescriptors, motifs: &[String]) -> String {
    let motif_text = motifs.join(" ").to_ascii_lowercase();
    if motif_text.contains("steroid") {
        return "steroid-like fused ring system".to_string();
    }
    if motif_text.contains("sugar") || motif_text.contains("tetrahydrofuran") {
        return "sugar-like oxygenated ring system".to_string();
    }
    if motif_text.contains("macrolide")
        || motif_text.contains("macrocycle")
        || motif_text.contains("lactone")
        || motif_text.contains("lactam")
    {
        return "macrolide-like oxygenated macrocycle".to_string();
    }
    if motif_text.contains("indole")
        || motif_text.contains("quinoline")
        || motif_text.contains("isoquinoline")
        || motif_text.contains("benzofuran")
        || motif_text.contains("benzothiophene")
        || motif_text.contains("quinoxaline")
        || motif_text.contains("purine")
    {
        return "fused heteroaromatic scaffold".to_string();
    }

    let rings = descriptors.ring_count.unwrap_or(0.0);
    let aromatic = descriptors.aromatic_ring_count.unwrap_or(0.0);
    let aliphatic = descriptors.aliphatic_ring_count.unwrap_or(0.0);
    let csp3 = descriptors.fraction_csp3.unwrap_or(0.0);

    if rings <= 0.0 {
        return "acyclic".to_string();
    }
    if aromatic >= 3.0 {
        return "polyaromatic scaffold".to_string();
    }
    if aromatic > 0.0 && aliphatic > 0.0 {
        return "mixed aromatic/aliphatic scaffold".to_string();
    }
    if aliphatic >= 2.0 || csp3 >= 0.5 {
        return "natural-product-like polycyclic scaffold".to_string();
    }
    "compact ring scaffold".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn np_score_rewards_sp3_rich_ringed_scaffolds() {
        let descriptors = RdkitDescriptors {
            fraction_csp3: Some(0.68),
            ring_count: Some(4.0),
            aromatic_ring_count: Some(0.0),
            aliphatic_ring_count: Some(2.0),
            tpsa: Some(72.0),
            clogp: Some(2.8),
            hetero_atoms: Some(6.0),
            rotatable_bonds: Some(5.0),
            hba: Some(6.0),
            hbd: Some(2.0),
            ..Default::default()
        };
        let assessment = assess_np_evidence(
            &descriptors,
            &["Steroid-like fused ring".to_string()],
            &["R/S".to_string()],
        );
        assert!(assessment.np_likeness > 1.0);
        assert_eq!(assessment.np_label, "strong NP-like");
        assert!(assessment.motif_context.contains("scaffold"));
    }
}
