//! Evidence assessment for natural-product (NP) originality.
//!
//! **Primary evidence** — the Ertl NP-likeness score — is computed in the
//! rdkit.js bridge using the open-data fragment-contribution model from:
//!
//! > Ertl, P., Roggo, S., & Schuffenhauer, A. (2008). "Natural Product-likeness
//! > Score and Its Application for Prioritization of Compound Libraries."
//! > *J. Chem. Inf. Model.*, 48, 68–74. DOI: 10.1021/ci700286x
//!
//! The open-source, open-data implementation and model file (`np_model.bin`)
//! are from:
//!
//! > Jayaseelan, K. V., Moreno, P., Truszkowski, A., Ertl, P., & Steinbeck, C.
//! > (2012). "Natural product-likeness score revisited: an open-source, open-data
//! > implementation." *BMC Bioinformatics*, 13, 106. DOI: 10.1186/1471-2105-13-106
//!
//! The model was trained on ~50 000 natural products (open databases) vs.
//! ~1 M drug-like molecules from ZINC.  Each Morgan-fingerprint (radius 2)
//! bit carries a log-probability-ratio contribution; the score is the sum of
//! contributions divided by the heavy-atom count, with log-compression beyond
//! ±4 to prevent score explosion.
//!
//! **Secondary evidence** — structural observations — uses only values that a
//! practising natural-product chemist would recognise as NP-typical, drawn from
//! the same Ertl papers and from the "escaping the *flatland*" literature
//! (Ertl 2003, *J. Am. Chem. Soc.* 125, 10353; Ertl & Schuppenhauer 2011).

#[cfg(target_arch = "wasm32")]
use crate::model::{ChemistCheck, MoleculeRow};
use crate::model::{DatasetMotifContext, RdkitDescriptors, RdkitMotifHit, normalized_source_class};

/// Result of assessing a single molecule against the evidence framework.
#[derive(Clone, Debug)]
pub struct EvidenceAssessment {
    /// Ertl NP-likeness score (range ≈ −5 to +5), or 0.0 when the model
    /// is not available.
    pub np_likeness: f64,
    /// Human-readable classification label derived from the score.
    pub np_label: String,
    /// Confidence in the score (fraction of Morgan bits found in the model).
    pub np_confidence: f64,
    /// Scaffold-family classification.
    pub ring_family: String,
    /// Evidence notes, each citing the underlying observation.
    pub evidence_notes: Vec<String>,
    /// One-line summary of the dataset-level motif context.
    pub motif_context: String,
}

/// Classification thresholds for the Ertl NP-likeness score.
///
/// Empirically grounded thresholds:
/// - **≥ 2.0**: Good NP evidence (high confidence natural product)
/// - **0.5–2.0**: Ambiguous NP signals (could be synthetic or semi-synthetic)
/// - **-1.0–0.5**: Bad/weak signals (predominantly synthetic features)
/// - **< -1.0**: Highly synthetic (strong negative signals—rare in real NPs)
pub fn np_likeness_label(score: f64) -> &'static str {
    if score >= 2.0 {
        "strong natural product"
    } else if score >= 0.5 {
        "NP-ambiguous"
    } else if score >= -1.0 {
        "weak NP signals"
    } else {
        "highly synthetic"
    }
}

/// Assess NP evidence for a single molecule.
///
/// * `np_score` / `np_confidence` come from the rdkit.js bridge (real Ertl
///   model).  When `None`, the score cannot be computed and structural
///   observations are the only available evidence.
/// * `substituents` is the list of Ertl (2022) top-2000 NP substituent labels
///   found in the molecule via substructure matching.
/// * `dataset_context` carries motif prevalence across the entire uploaded
///   set so that per-row notes can flag dataset-common scaffolds.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::uninlined_format_args)]
pub fn assess_np_evidence(
    descriptors: &RdkitDescriptors,
    motifs: &[String],
    motif_hits: &[RdkitMotifHit],
    stereo_tags: &[String],
    np_score: Option<f64>,
    np_confidence: Option<f64>,
    dataset_context: &DatasetMotifContext,
    lotus_scaffolds: &[String],
) -> EvidenceAssessment {
    let ring_family = classify_ring_family(descriptors, motifs);
    let mut notes = Vec::new();
    let np_core_hits = count_core_np_motifs(motifs);
    let scaffold_hits = count_scaffold_hits(motif_hits);
    let decoration_hits = count_decoration_motifs(motifs);
    let natural_hits = count_source_hits(motif_hits, "natural");
    let synthetic_hits = count_source_hits(motif_hits, "synthetic");
    let unknown_hits = count_source_hits(motif_hits, "unknown");
    let kingdom_enriched_hits = count_kingdom_enriched_hits(motif_hits);
    let natural_only_hits = natural_hits.saturating_sub(kingdom_enriched_hits);
    let kingdom_support = kingdom_enriched_hits > 0;
    let score = np_score.unwrap_or(0.0);
    let confidence = np_confidence.unwrap_or(0.0);

    // ── Primary evidence: real Ertl NP-likeness score ──────────────────
    if let Some(s) = np_score {
        notes.push(format!(
            "Ertl NP-likeness score: {s:+.2} (model confidence {:.0}%) — \
             Ertl et al., J. Chem. Inf. Model. 2008, 48, 68",
            confidence * 100.0
        ));
        if confidence < 0.5 {
            notes.push(format!(
                "Low model coverage ({:.0}%) — Ertl score is \
                 unreliable for unusual fragments",
                confidence * 100.0
            ));
        }
    } else {
        notes.push("Ertl fragment model not loaded — NP-likeness score unavailable".into());
    }

    // ── LOTUS 1-percent scaffold evidence (Rutz et al. mortar fragmentation) ──
    if !lotus_scaffolds.is_empty() {
        notes.push(format!(
            "✓ {} LOTUS 1% scaffold(s) matched — Rutz et al. mortar \
             fragmentation; scaffolds appearing in >1% of LOTUS compounds",
            lotus_scaffolds.len()
        ));
    }

    // Stereochemical complexity — stereocentres are hallmarks of enzymatic
    // biosynthesis; synthetic libraries average <1 stereocentre per molecule.
    // ── Stereochemistry signal ─────────────────────────────────────────
    // Stereochemistry is enriched in natural products due to enzymatic
    // biosynthesis. Presence is a positive signal; absence in 2D SMILES is
    // neutral (MS annotation pipelines often strip stereo).
    if !stereo_tags.is_empty() {
        notes.push(format!(
            "✓ {} stereochemical center(s) — consistent with enzymatic origin",
            stereo_tags.len()
        ));
    }

    // ── NP motif presence ─────────────────────────────────────────────
    if np_core_hits > 0 {
        notes.push(format!(
            "✓ {} Ertl-style NP motif(s): {}",
            np_core_hits,
            motifs
                .iter()
                .filter(|m| is_known_np_motif(m))
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if scaffold_hits > 0 && np_core_hits == 0 {
        notes.push(format!(
            "○ {scaffold_hits} structural support motif(s) — supportive only, not Ertl-enriched by itself",
        ));
    }

    if kingdom_enriched_hits > 0 || natural_only_hits > 0 || synthetic_hits > 0 || unknown_hits > 0
    {
        let total = kingdom_enriched_hits + natural_only_hits + synthetic_hits + unknown_hits;
        let natural_ratio = if total > 0 {
            (kingdom_enriched_hits + natural_only_hits) as f64 / total as f64
        } else {
            0.0
        };
        let synthetic_ratio = if total > 0 {
            synthetic_hits as f64 / total as f64
        } else {
            0.0
        };
        notes.push(format!(
            "motif balance: {kingdom_enriched_hits} kingdom-enriched / {natural_only_hits} natural-only / {synthetic_hits} synthetic / {unknown_hits} unclassified ({natural_ratio:.0}% natural, {synthetic_ratio:.0}% synthetic)"
        ));
    }

    if decoration_hits > 0 && np_core_hits == 0 {
        notes.push(format!(
            "○ {decoration_hits} decoration motif(s) — supportive side-chain chemistry, but weak NP evidence alone",
        ));
    }

    // ── Dataset-level context ──────────────────────────────────────────
    let dataset_shared = motifs
        .iter()
        .filter(|m| {
            dataset_context
                .motif_counts
                .get(*m)
                .is_some_and(|&c| c >= dataset_context.common_threshold)
        })
        .count();
    if dataset_shared > 0 {
        notes.push(format!(
            "✓ {dataset_shared} motif(s) shared with dataset — possible family"
        ));
    }

    // ── Motif context summary ──────────────────────────────────────────
    let motif_context = if np_core_hits > 0 && scaffold_hits > 0 {
        "Ertl-style NP core with structural support".to_string()
    } else if np_core_hits > 0 {
        "Ertl-style NP core motifs".to_string()
    } else if scaffold_hits > 0 && kingdom_support {
        "kingdom-enriched structural motifs".to_string()
    } else if scaffold_hits > 0 && natural_only_hits > 0 {
        "natural-only structural motifs".to_string()
    } else if scaffold_hits > 0 {
        "structural support motifs".to_string()
    } else if kingdom_support && kingdom_enriched_hits > synthetic_hits {
        "kingdom-enriched motifs".to_string()
    } else if natural_only_hits > synthetic_hits && natural_only_hits > 0 {
        "natural-leaning motifs".to_string()
    } else if synthetic_hits > natural_hits && synthetic_hits > 0 {
        "synthetic-leaning motifs".to_string()
    } else if decoration_hits > 0 {
        "decoration motifs only".to_string()
    } else {
        "no structural motifs detected".to_string()
    };

    let np_label = np_likeness_label(score).to_string();

    EvidenceAssessment {
        np_likeness: score,
        np_label,
        np_confidence: confidence,
        ring_family,
        evidence_notes: notes,
        motif_context,
    }
}

/// Verdict string shown prominently in the UI.
#[cfg(target_arch = "wasm32")]
pub fn verdict_for_row(row: &crate::model::MoleculeRow) -> String {
    if let Some(err) = row.error.as_deref() {
        return format!("⚠ {err}");
    }

    let has_lotus = !row.lotus_taxa.is_empty();
    let has_pubchem = !row.pubchem_cids.is_empty();
    let score = row.np_likeness;

    if !row.np_score_available {
        if has_lotus {
            return "🌿 LOTUS-backed evidence".to_string();
        }
        return "⚠ Ertl model not loaded".to_string();
    }

    if has_lotus && score >= 2.0 {
        return format!("🌿 LOTUS + strong NP score (Ertl {score:+.2})");
    }
    if has_lotus && score >= 0.5 {
        return format!("🌿 LOTUS hit (Ertl {score:+.2})");
    }
    if has_lotus {
        return format!("🌿 LOTUS-backed but weak score (Ertl {score:+.2})");
    }

    // ── Very weak/synthetic signals always stay yellow/red ──────────────
    if score <= -1.0 {
        return format!("⚠ Weak NP signals (Ertl {score:+.2})");
    }

    // ── PubChem hit evaluation ──────────────────────────────────────────
    // PubChem is a WEAK signal on its own; strong structural evidence required
    if has_pubchem {
        let np_motif_count = count_core_np_motifs(&row.motifs);
        let has_np_scaffold = row.ring_family.to_ascii_lowercase().contains("polycyclic")
            || row.ring_family.to_ascii_lowercase().contains("steroid")
            || row.ring_family.to_ascii_lowercase().contains("sugar")
            || row.ring_family.to_ascii_lowercase().contains("macrolide")
            || row.ring_family.to_ascii_lowercase().contains("flavonoid")
            || row
                .ring_family
                .to_ascii_lowercase()
                .contains("fused heteroaromatic");
        let structural_support = np_motif_count >= 1 || has_np_scaffold;
        let natural_weight = row
            .motif_hits
            .iter()
            .filter(|hit| normalized_source_class(&hit.source_class) == "natural")
            .count();
        let kingdom_enriched_hits = count_kingdom_enriched_hits(&row.motif_hits);
        let synthetic_weight = row
            .motif_hits
            .iter()
            .filter(|hit| normalized_source_class(&hit.source_class) == "synthetic")
            .count();
        let kingdom_support = kingdom_enriched_hits > 0;

        // ONLY go green if score is strong AND structure looks chemically NP-like.
        if score >= 2.0
            && (structural_support || natural_weight >= synthetic_weight || kingdom_support)
        {
            return format!("🌿 PubChem + strong NP evidence (Ertl {score:+.2})");
        }

        if score >= 2.0 {
            return format!("🌿 Strong NP score (Ertl {score:+.2})");
        }

        // Score 1.0-2.0 (ambiguous) with supporting structural cues → blue
        if score >= 1.0 && structural_support {
            return format!("📚 PubChem with NP motifs (Ertl {score:+.2})");
        }

        // Any PubChem with score 0.5-1.0 → yellow (needs citation)
        if score >= 0.5 {
            return format!("👃 Citation needed — PubChem hit (Ertl {score:+.2})");
        }

        // Score -1.0 to 0.5 → weak signals
        return format!("📚 PubChem — weak NP signals (Ertl {score:+.2})");
    }

    // Not in databases — structural evidence only
    let has_np_scaffold = row.ring_family.to_ascii_lowercase().contains("polycyclic")
        || row.ring_family.to_ascii_lowercase().contains("steroid")
        || row.ring_family.to_ascii_lowercase().contains("sugar")
        || row.ring_family.to_ascii_lowercase().contains("macrolide")
        || row.ring_family.to_ascii_lowercase().contains("flavonoid")
        || row
            .ring_family
            .to_ascii_lowercase()
            .contains("fused heteroaromatic");
    let has_np_motifs = count_core_np_motifs(&row.motifs) > 0;
    let natural_hits = row
        .motif_hits
        .iter()
        .filter(|hit| normalized_source_class(&hit.source_class) == "natural")
        .count();
    let kingdom_enriched_hits = row
        .motif_hits
        .iter()
        .filter(|hit| {
            normalized_source_class(&hit.source_class) == "natural" && !hit.kingdoms.is_empty()
        })
        .count();
    let natural_only_hits = natural_hits.saturating_sub(kingdom_enriched_hits);
    let synthetic_hits = row
        .motif_hits
        .iter()
        .filter(|hit| normalized_source_class(&hit.source_class) == "synthetic")
        .count();
    let natural_weight = natural_only_hits + kingdom_enriched_hits * 2;
    let synthetic_weight = synthetic_hits
        + row
            .motif_hits
            .iter()
            .filter(|hit| normalized_source_class(&hit.source_class) == "unknown")
            .count();
    let kingdom_support = kingdom_enriched_hits > 0;
    if score >= 2.0 && (has_np_scaffold || natural_weight >= synthetic_weight || kingdom_support) {
        return format!("🌿 Likely novel NP (Ertl {score:+.2})");
    }
    if score >= 1.0 && has_np_motifs && has_np_scaffold {
        return format!("🌿 Likely novel NP (Ertl {score:+.2})");
    }
    if score >= 1.0 && kingdom_support && kingdom_enriched_hits >= synthetic_weight {
        return format!("🌿 Kingdom-enriched motifs (Ertl {score:+.2})");
    }
    if score >= 1.0 && natural_only_hits > synthetic_weight && natural_only_hits > 0 {
        return format!("🌿 Natural-leaning motifs (Ertl {score:+.2})");
    }
    if score >= 1.0 && synthetic_weight > natural_weight && synthetic_hits > 0 {
        return format!("⚠ Synthetic-leaning motifs (Ertl {score:+.2})");
    }
    if score >= 2.0 {
        return format!("Strong NP score (Ertl {score:+.2})");
    }
    if score <= -1.0 {
        return format!("⚠ Weak NP signals (Ertl {score:+.2})");
    }
    format!("👃 Citation needed (Ertl {score:+.2})")
}

/// Machine-readable category for CSV export — strips emojis and
/// normalises to "likely", "neutral", "caution", "skeptical", or "fishy".
pub fn verdict_category(verdict: &str) -> &'static str {
    let l = verdict.to_ascii_lowercase();

    // RED — Highly synthetic / negative signals / fishy (check first!)
    if l.contains("highly synthetic") || l.contains("smells fishy") {
        return "fishy";
    }

    // GREEN — High NP confidence (LOTUS or strong structural + Ertl score)
    if l.contains("lotus") && !l.contains("weak") {
        return "likely";
    }
    if l.contains("likely hit") {
        return "likely";
    }
    if l.contains("likely novel") {
        return "likely";
    }
    if l.contains("pubchem + strong") {
        return "likely";
    }
    if l.contains("strong np score") && !l.contains("weak") {
        return "likely";
    }

    // YELLOW — Skeptical / ambiguous (needs citation, uncertain origin)
    if l.contains("citation needed") {
        return "skeptical";
    }

    // BLUE — Moderate NP confidence (PubChem with some NP features)
    if l.contains("pubchem with") || l.contains("pubchem + np") {
        return "neutral";
    }

    // RED — Low/weak NP confidence
    if l.contains("weak np signals")
        || (l.contains("pubchem") && l.contains("weak"))
        || (l.contains("ertl") && (l.contains("−1") || l.contains("-1")))
    {
        return "caution";
    }

    "neutral"
}

/// Classify the core scaffold family using motif SMARTS matches and
/// descriptor-based heuristics.
pub fn classify_ring_family(descriptors: &RdkitDescriptors, motifs: &[String]) -> String {
    let motif_text = motifs.join(" ").to_ascii_lowercase();

    // Steroids — tetracyclic fused ring system with characteristic
    // cyclopentanoperhydrophenanthrene core.
    if motif_text.contains("steroid") {
        return "steroid-like fused ring system".to_string();
    }
    // Monosaccharides and THF rings — common in glycosylated NPs.
    if motif_text.contains("sugar") || motif_text.contains("tetrahydrofuran") {
        return "sugar-like oxygenated ring system".to_string();
    }
    // Macrocycles, lactones, lactams — hallmark macrocyclic NP scaffolds.
    if motif_text.contains("macrolide")
        || motif_text.contains("macrocycle")
        || motif_text.contains("lactone")
        || motif_text.contains("lactam")
    {
        return "macrolide-like oxygenated macrocycle".to_string();
    }
    // Benzopyran, flavone, flavonoid — plant secondary metabolite cores.
    if motif_text.contains("flavone") || motif_text.contains("flavonoid") {
        return "flavonoid-like scaffold".to_string();
    }
    // N-heteroaromatic scaffolds — common in both NPs and synthetic drugs.
    if motif_text.contains("indole")
        || motif_text.contains("quinoline")
        || motif_text.contains("isoquinoline")
        || motif_text.contains("benzofuran")
        || motif_text.contains("benzothiophene")
        || motif_text.contains("quinoxaline")
        || motif_text.contains("purine")
        || motif_text.contains("chromone")
        || motif_text.contains("coumarin")
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
    // Three or more aromatic rings → polyaromatic (PAHs, not typical of NPs).
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

/// Quick-check audit that a natural-product chemist would run when
/// eyeballing a structure.
///
/// Each check returns a `ChemistCheck` with status `"pass"`, `"warn"`, or
/// `"fail"`.  The checks are:
///
/// 1. **NP-likeness** — the Ertl score is the statistical ground truth;
///    scores above 0.5 are supportive of NP origin.
/// 2. **Skeleton** — scaffolds that match known natural-product ring systems
///    (steroids, sugars, macrocycles, flavonoids, etc.) score positively;
///    pure polyaromatic systems are a red flag.
/// 3. **Database** — presence in LOTUS (curated NP database) or `PubChem`
///    provides orthogonal evidence.
///
/// **Stereochemistry is deliberately NOT checked** — 2D SMILES from mass
/// spectrometry annotation pipelines may not preserve stereochemical
/// information, so the absence of stereo tags is not a reliable negative
/// signal.
#[cfg(target_arch = "wasm32")]
pub fn chemist_checks(row: &MoleculeRow) -> Vec<ChemistCheck> {
    let mut checks: Vec<ChemistCheck> = Vec::with_capacity(3);

    // 1 — NP-likeness score
    if !row.np_score_available {
        checks.push(ChemistCheck {
            name: "NP-likeness",
            status: "warn",
            detail: "Ertl model not loaded".into(),
        });
    } else if row.np_likeness >= 0.5 {
        checks.push(ChemistCheck {
            name: "NP-likeness",
            status: "pass",
            detail: format!("Ertl score {:+.2}", row.np_likeness),
        });
    } else if row.np_likeness > -0.5 {
        checks.push(ChemistCheck {
            name: "NP-likeness",
            status: "warn",
            detail: format!("Ertl score {:+.2} — borderline", row.np_likeness),
        });
    } else {
        checks.push(ChemistCheck {
            name: "NP-likeness",
            status: "fail",
            detail: format!("Ertl score {:+.2} — synthetic-leaning", row.np_likeness),
        });
    }

    // 2 — Skeleton / scaffold classification
    let family = row.ring_family.to_ascii_lowercase();
    let is_polyaromatic = family.contains("polyaromatic");
    let is_np_skeleton = family.contains("polycyclic")
        || family.contains("steroid")
        || family.contains("sugar")
        || family.contains("macrolide")
        || family.contains("flavonoid")
        || family.contains("heteroaromatic");
    if is_polyaromatic {
        checks.push(ChemistCheck {
            name: "Skeleton",
            status: "fail",
            detail: row.ring_family.clone(),
        });
    } else if is_np_skeleton {
        checks.push(ChemistCheck {
            name: "Skeleton",
            status: "pass",
            detail: row.ring_family.clone(),
        });
    } else {
        checks.push(ChemistCheck {
            name: "Skeleton",
            status: "warn",
            detail: row.ring_family.clone(),
        });
    }

    // 3 — Database presence
    let has_lotus = !row.lotus_taxa.is_empty();
    let has_pubchem = !row.pubchem_cids.is_empty();
    if has_lotus && has_pubchem {
        checks.push(ChemistCheck {
            name: "Database",
            status: "pass",
            detail: "LOTUS + PubChem".into(),
        });
    } else if has_lotus {
        checks.push(ChemistCheck {
            name: "Database",
            status: "pass",
            detail: "LOTUS taxa".into(),
        });
    } else if has_pubchem {
        checks.push(ChemistCheck {
            name: "Database",
            status: "warn",
            detail: "PubChem only".into(),
        });
    } else {
        checks.push(ChemistCheck {
            name: "Database",
            status: "fail",
            detail: "Not in LOTUS or PubChem".into(),
        });
    }

    checks
}

/// Motif labels that are known to be enriched in natural products, based on
/// Ertl & Schuhmann (J. Nat. Prod. 2019, Vol. 82, 1258-1263,
/// DOI 10.1021/acs.jnatprod.8b01022)
/// and Wetzel et al. (CHIMIA 2007, DOI 10.2533/chimia.2007.355).
///
/// Used to highlight motifs that are characteristic of NP biosynthesis
/// in the UI.
pub fn is_known_np_motif(label: &str) -> bool {
    let l = label.to_ascii_lowercase();
    // NP scaffold classes
    l.contains("steroid")
        || l.contains("sugar")
        || l.contains("macrolide")
        || l.contains("macrocycle")
        || l.contains("lactone")
        || l.contains("lactam")
        || l.contains("flavone")
        || l.contains("flavonoid")
        || l.contains("indole")
        || l.contains("quinoline")
        || l.contains("isoquinoline")
        || l.contains("benzofuran")
        || l.contains("benzothiophene")
        || l.contains("quinoxaline")
        || l.contains("purine")
        || l.contains("chromone")
        || l.contains("coumarin")
        || l.contains("tetrahydrofuran")
        || l.contains("tetrahydropyran")
        || l.contains("piperidine")
        || l.contains("piperazine")
        || l.contains("morpholine")
}

fn count_core_np_motifs(motifs: &[String]) -> usize {
    motifs.iter().filter(|m| is_known_np_motif(m)).count()
}

fn count_scaffold_hits(hits: &[RdkitMotifHit]) -> usize {
    hits.iter()
        .filter(|hit| is_scaffold_motif(&hit.label))
        .count()
}

fn count_source_hits(hits: &[RdkitMotifHit], source_class: &str) -> usize {
    hits.iter()
        .filter(|hit| normalized_source_class(&hit.source_class) == source_class)
        .count()
}

fn count_kingdom_enriched_hits(hits: &[RdkitMotifHit]) -> usize {
    hits.iter()
        .filter(|hit| {
            normalized_source_class(&hit.source_class) == "natural" && !hit.kingdoms.is_empty()
        })
        .count()
}

fn count_decoration_motifs(motifs: &[String]) -> usize {
    motifs.iter().filter(|m| is_decoration_motif(m)).count()
}

/// Scaffold motifs are ring systems or cores characteristic of natural-product
/// scaffold classes.
pub fn is_scaffold_motif(label: &str) -> bool {
    let l = label.to_ascii_lowercase();
    l.contains("ring")
        || l.contains("steroid")
        || l.contains("sugar")
        || l.contains("macrocycle")
        || l.contains("macrolide")
        || l.contains("lactone")
        || l.contains("lactam")
        || l.contains("flavone")
        || l.contains("flavonoid")
        || l.contains("indole")
        || l.contains("quinoline")
        || l.contains("isoquinoline")
        || l.contains("benzofuran")
        || l.contains("benzothiophene")
        || l.contains("quinoxaline")
        || l.contains("purine")
        || l.contains("chromone")
        || l.contains("coumarin")
        || l.contains("morpholine")
        || l.contains("piperidine")
        || l.contains("piperazine")
        || l.contains("tetrahydrofuran")
        || l.contains("tetrahydropyran")
        || l.contains("cyclohexane")
}

/// Decoration motifs are functional groups or side-chain fragments.
fn is_decoration_motif(label: &str) -> bool {
    let l = label.to_ascii_lowercase();
    l.contains("aldehyde")
        || l.contains("ketone")
        || l.contains("carboxylic acid")
        || l.contains("ester")
        || l.contains("amide")
        || l.contains("carbamate")
        || l.contains("urea")
        || l.contains("sulfonamide")
        || l.contains("sulfone")
        || l.contains("sulfoxide")
        || l.contains("alcohol")
        || l.contains("phenol")
        || l.contains("ether")
        || l.contains("methoxy")
        || l.contains("amine")
        || l.contains("halogen")
        || l.contains("nitrile")
        || l.contains("nitro")
        || l.contains("thiol")
        || l.contains("phosphate")
        || l.contains("acetal")
        || l.contains("epoxide")
        || l.contains("allyl")
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn empty_descriptors() -> RdkitDescriptors {
        RdkitDescriptors::default()
    }

    fn empty_dataset_context() -> DatasetMotifContext {
        DatasetMotifContext::default()
    }

    fn motif_hit(label: &str, source_class: &str, kingdom: &str) -> RdkitMotifHit {
        RdkitMotifHit {
            label: label.to_string(),
            source_class: source_class.to_string(),
            kingdom: kingdom.to_string(),
            kingdoms: if kingdom.is_empty() {
                Vec::new()
            } else {
                vec![kingdom.to_string()]
            },
        }
    }

    #[test]
    fn label_thresholds_match_ertl_distribution() {
        assert_eq!(np_likeness_label(5.0), "strong natural product");
        assert_eq!(np_likeness_label(2.0), "strong natural product");
        assert_eq!(np_likeness_label(0.8), "NP-ambiguous");
        assert_eq!(np_likeness_label(0.5), "NP-ambiguous");
        assert_eq!(np_likeness_label(0.0), "weak NP signals");
        assert_eq!(np_likeness_label(-0.5), "weak NP signals");
        assert_eq!(np_likeness_label(-1.0), "weak NP signals");
        assert_eq!(np_likeness_label(-1.5), "highly synthetic");
        assert_eq!(np_likeness_label(-5.0), "highly synthetic");
    }

    #[test]
    fn real_ertl_score_is_used() {
        let desc = RdkitDescriptors {
            fraction_csp3: Some(0.68),
            ring_count: Some(4.0),
            aromatic_ring_count: Some(0.0),
            aliphatic_ring_count: Some(2.0),
        };
        let assessment = assess_np_evidence(
            &desc,
            &["Steroid-like fused ring".to_string()],
            &[],
            &["R/S".to_string(), "R/S".to_string()],
            Some(3.42),
            Some(0.75),
            &empty_dataset_context(),
            &[],
        );
        assert!((assessment.np_likeness - 3.42).abs() < 1e-9);
        assert_eq!(assessment.np_label, "strong natural product");
        assert!(!assessment.ring_family.is_empty());
        assert!(assessment.np_confidence > 0.5);
        assert!(
            assessment
                .evidence_notes
                .iter()
                .any(|n| n.contains("Ertl NP-likeness score"))
        );
        assert!(
            assessment
                .evidence_notes
                .iter()
                .any(|n| n.contains("stereochemical"))
        );
    }

    #[test]
    fn no_made_up_tpsa_rule() {
        let desc = RdkitDescriptors::default();
        let assessment = assess_np_evidence(
            &desc,
            &[],
            &[],
            &[],
            Some(1.5),
            Some(0.8),
            &empty_dataset_context(),
            &[],
        );
        // No "polar enough" note should exist — that was a hallucinated rule.
        assert!(
            assessment
                .evidence_notes
                .iter()
                .all(|n| !n.contains("polar enough"))
        );
        assert!(
            assessment
                .evidence_notes
                .iter()
                .all(|n| !n.contains("TPSA"))
        );
    }

    #[test]
    fn no_made_up_clogp_rule() {
        let desc = RdkitDescriptors::default();
        let assessment = assess_np_evidence(
            &desc,
            &[],
            &[],
            &[],
            Some(-1.5),
            Some(0.9),
            &empty_dataset_context(),
            &[],
        );
        // No clogP-based notes should exist — those were hallucinated rules.
        assert!(
            assessment
                .evidence_notes
                .iter()
                .all(|n| !n.contains("clogp") && !n.contains("ClogP") && !n.contains("lipophilic"))
        );
    }

    #[test]
    fn model_unavailable_is_handled() {
        let assessment = assess_np_evidence(
            &empty_descriptors(),
            &["Flavonoid core".to_string()],
            &[],
            &[],
            None,
            None,
            &empty_dataset_context(),
            &[],
        );
        assert!((assessment.np_likeness - 0.0).abs() < 1e-9);
        assert_eq!(assessment.np_label, "weak NP signals");
        assert!(!assessment.ring_family.is_empty());
        assert!(
            assessment
                .evidence_notes
                .iter()
                .any(|n| n.contains("model not loaded"))
        );
    }

    #[test]
    fn dataset_common_motif_is_noted() {
        let desc = empty_descriptors();
        let mut ctx = DatasetMotifContext {
            motif_counts: HashMap::new(),
            common_threshold: 2,
        };
        ctx.motif_counts.insert("Flavonoid core".to_string(), 3);
        ctx.motif_counts.insert("Aldehyde".to_string(), 1);

        let assessment = assess_np_evidence(
            &desc,
            &["Flavonoid core".to_string(), "Aldehyde".to_string()],
            &[],
            &[],
            Some(-0.5),
            Some(0.5),
            &ctx,
            &[],
        );
        assert!(
            assessment
                .evidence_notes
                .iter()
                .any(|n| n.contains("motif") || n.contains("Flavonoid"))
        );
    }

    #[test]
    fn scaffold_hit_is_counted_as_structure() {
        let assessment = assess_np_evidence(
            &empty_descriptors(),
            &["geminal dimethyl".to_string()],
            &[motif_hit("geminal dimethyl", "natural", "plants")],
            &[],
            Some(0.2),
            Some(0.9),
            &empty_dataset_context(),
            &[],
        );
        assert_ne!(assessment.motif_context, "no structural motifs detected");
        assert!(
            assessment
                .evidence_notes
                .iter()
                .any(|n| n.contains("motif balance"))
        );
    }

    #[test]
    fn kingdom_enriched_hits_are_distinct() {
        let assessment = assess_np_evidence(
            &empty_descriptors(),
            &["geminal dimethyl".to_string(), "CC=C".to_string()],
            &[
                motif_hit("geminal dimethyl", "natural", "plants"),
                motif_hit("CC=C", "natural", ""),
            ],
            &[],
            Some(1.1),
            Some(0.9),
            &empty_dataset_context(),
            &[],
        );
        assert!(
            assessment
                .evidence_notes
                .iter()
                .any(|n| n.contains("kingdom-enriched"))
        );
    }

    #[test]
    fn ring_family_still_functions() {
        let desc = RdkitDescriptors {
            ring_count: Some(4.0),
            aromatic_ring_count: Some(0.0),
            aliphatic_ring_count: Some(4.0),
            fraction_csp3: Some(0.75),
        };
        let family = classify_ring_family(&desc, &[]);
        assert_eq!(family, "natural-product-like polycyclic scaffold");

        let desc2 = RdkitDescriptors {
            ring_count: Some(3.0),
            aromatic_ring_count: Some(3.0),
            aliphatic_ring_count: Some(0.0),
            fraction_csp3: Some(0.10),
        };
        let family2 = classify_ring_family(&desc2, &[]);
        assert_eq!(family2, "polyaromatic scaffold");
    }

    #[test]
    fn flavor_classification_via_motif() {
        let family = classify_ring_family(&empty_descriptors(), &["Flavone ring".to_string()]);
        assert_eq!(family, "flavonoid-like scaffold");

        let family2 = classify_ring_family(&empty_descriptors(), &["Indole ring".to_string()]);
        assert_eq!(family2, "fused heteroaromatic scaffold");

        let family3 = classify_ring_family(
            &empty_descriptors(),
            &["Sugar-like oxygen ring".to_string()],
        );
        assert_eq!(family3, "sugar-like oxygenated ring system");
    }

    #[test]
    fn decoration_motifs_stay_neutral() {
        assert!(!is_known_np_motif("Methoxy"));
        assert!(is_decoration_motif("Methoxy"));
        assert!(!is_scaffold_motif("Methoxy"));
    }

    #[test]
    fn verdict_category_high_quality_pubchem() {
        // High-quality PubChem hit should be "likely", not "caution"
        let verdict = "🌿 PubChem + strong NP evidence — Ertl score +1.50 with 3 NP substituent(s) + 2 NP motif(s).";
        assert_eq!(verdict_category(verdict), "likely");
    }

    #[test]
    fn verdict_category_moderate_quality_pubchem() {
        // Moderate-quality PubChem hit should be "neutral"
        let verdict =
            "📚 PubChem hit with NP signals — Ertl score +0.80, 2 substituent(s), 1 NP motif(s).";
        assert_eq!(verdict_category(verdict), "neutral");
    }

    #[test]
    fn verdict_category_weak_pubchem() {
        // Weak PubChem hit should be "caution"
        let verdict = "📚 PubChem hit — weak NP evidence (Ertl score +0.26).";
        assert_eq!(verdict_category(verdict), "caution");
    }

    #[test]
    fn verdict_category_lotus_backed() {
        let verdict = "🌿 LOTUS-backed (Ertl score +1.23).";
        assert_eq!(verdict_category(verdict), "likely");
    }

    #[test]
    fn verdict_category_novel_candidate() {
        let verdict =
            "🌿 Likely hit — strong NP-likeness (+3.43) + NP-like scaffold, not yet in databases.";
        assert_eq!(verdict_category(verdict), "likely");
    }

    #[test]
    fn verdict_category_fishy() {
        let verdict = "👃 Smells fishy (Ertl score -1.23). Citation needed.";
        assert_eq!(verdict_category(verdict), "fishy");
    }
}
