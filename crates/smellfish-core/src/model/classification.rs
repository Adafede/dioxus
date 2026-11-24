// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

//! Scaffold ring-family, motif-context, and verdict classification enums.

use std::fmt;

/// Scaffold ring-system classification derived from motif labels or descriptors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum RingFamily {
    /// Tetracyclic steroid-like core.
    Steroid,
    /// Glycoside / oxygenated THF-like sugar core.
    Sugar,
    /// Macrocyclic / lactone / lactam core.
    Macrolide,
    /// Flavone / flavonoid plant core.
    Flavonoid,
    /// Fused heteroaromatic system.
    Heteroaromatic,
    /// Polyaromatic system (`>= 3` aromatic rings).
    Polyaromatic,
    /// Mixed aromatic and aliphatic ring system.
    MixedAromaticAliphatic,
    /// Polycyclic aliphatic / `sp3`-rich system.
    Polycyclic,
    /// Single or small ring system.
    CompactRing,
    /// Zero rings.
    #[default]
    Acyclic,
}

impl RingFamily {
    /// Human-readable scaffold description matching the legacy display labels.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Steroid => "steroid-like fused ring system",
            Self::Sugar => "sugar-like oxygenated ring system",
            Self::Macrolide => "macrolide-like oxygenated macrocycle",
            Self::Flavonoid => "flavonoid-like scaffold",
            Self::Heteroaromatic => "fused heteroaromatic scaffold",
            Self::Polyaromatic => "polyaromatic scaffold",
            Self::MixedAromaticAliphatic => "mixed aromatic/aliphatic scaffold",
            Self::Polycyclic => "natural-product-like polycyclic scaffold",
            Self::CompactRing => "compact ring scaffold",
            Self::Acyclic => "acyclic",
        }
    }

    /// Returns `true` when the scaffold is classified as polyaromatic.
    #[must_use]
    pub const fn is_polyaromatic(self) -> bool {
        matches!(self, Self::Polyaromatic)
    }

    /// Returns `true` for natural-product-associated ring families.
    #[must_use]
    pub const fn is_np_skeleton(self) -> bool {
        matches!(
            self,
            Self::Steroid
                | Self::Sugar
                | Self::Macrolide
                | Self::Flavonoid
                | Self::Heteroaromatic
                | Self::Polycyclic
        )
    }

    /// Display emoji for the scaffold line in the UI.
    #[must_use]
    pub const fn emoji(self) -> &'static str {
        if self.is_polyaromatic() {
            "\u{26a0}"
        } else if self.is_np_skeleton() {
            "\u{1f33f}"
        } else {
            "\u{2014}"
        }
    }
}

impl fmt::Display for RingFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Summary of the structural motif balance for a molecule.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum MotifContext {
    /// Both Ertl core NP motifs and structural support rings.
    ErtlCoreWithSupport,
    /// Ertl core NP motifs without separate support ring hits.
    ErtlCore,
    /// Scaffold hits accompanied by kingdom taxonomy enrichment.
    KingdomEnrichedStructural,
    /// Scaffold hits accompanied by natural-source motif balance.
    NaturalStructural,
    /// Scaffold hits without kingdom or natural-only majority.
    StructuralSupport,
    /// Kingdom-enriched motifs outnumbering synthetic hits.
    KingdomEnriched,
    /// Natural-only motifs outnumbering synthetic hits.
    NaturalLeaning,
    /// Synthetic hits outnumbering natural hits.
    SyntheticLeaning,
    /// Side-chain / decoration hits only.
    DecorationOnly,
    /// No structural motifs matched.
    #[default]
    NoneDetected,
}

impl MotifContext {
    /// Returns `true` when any structural motif signal was detected.
    #[must_use]
    pub const fn is_detected(self) -> bool {
        !matches!(self, Self::NoneDetected)
    }

    /// Human-readable one-line motif summary.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ErtlCoreWithSupport => "Ertl-style NP core with structural support",
            Self::ErtlCore => "Ertl-style NP core motifs",
            Self::KingdomEnrichedStructural => "kingdom-enriched structural motifs",
            Self::NaturalStructural => "natural-only structural motifs",
            Self::StructuralSupport => "structural support motifs",
            Self::KingdomEnriched => "kingdom-enriched motifs",
            Self::NaturalLeaning => "natural-leaning motifs",
            Self::SyntheticLeaning => "synthetic-leaning motifs",
            Self::DecorationOnly => "decoration motifs only",
            Self::NoneDetected => "no structural motifs detected",
        }
    }
}

impl fmt::Display for MotifContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Coarse verdict category used directly by both UI styling and CSV export.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VerdictCategory {
    /// Strong / DB-corroborated NP candidate.
    Likely,
    /// Neutral / informational outcome.
    #[default]
    Neutral,
    /// Borderline or synthetic-leaning caution flag.
    Caution,
    /// Requires supporting literature or structural evidence.
    Skeptical,
    /// Strongly synthetic (`Ertl <= -2.0`).
    Fishy,
}

impl VerdictCategory {
    /// Machine-readable CSV category label.
    #[must_use]
    pub const fn csv_label(self) -> &'static str {
        match self {
            Self::Likely => "likely",
            Self::Neutral => "neutral",
            Self::Caution => "caution",
            Self::Skeptical => "skeptical",
            Self::Fishy => "fishy",
        }
    }

    /// CSS class name paired with `.verdict` in the web UI.
    #[must_use]
    pub const fn css_class(self) -> &'static str {
        match self {
            Self::Likely => "verdict-likely",
            Self::Neutral => "verdict-neutral",
            Self::Caution => "verdict-caution",
            Self::Skeptical => "verdict-skeptical",
            Self::Fishy => "verdict-fishy",
        }
    }
}

/// Typed screening verdict combining category and human-readable explanation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Verdict {
    /// Category driving badge color and CSV export.
    pub category: VerdictCategory,
    /// Human-facing verdict line.
    pub message: String,
}

impl Verdict {
    /// Constructs a verdict from a category and message.
    #[must_use]
    pub fn new(category: VerdictCategory, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }

    /// Wraps a structure/inspection error as a neutral warning verdict.
    #[must_use]
    pub fn error(err: &str) -> Self {
        Self::new(VerdictCategory::Neutral, format!("⚠ {err}"))
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
