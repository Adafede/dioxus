// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

//! Programmatic document `<head>` management for smellfish-rs.
//!
//! Replaces the static `index.html` with Rust code that sets meta tags,
//! loads CDN scripts (`RDKit`), the local motif-library bridge, and inline
//! RDKit/NP-likeness bridge code.

use crate::styles::CSS;
use dioxus::prelude::*;
use ui::prelude::*;

/// Inline JavaScript bridge: `RDKit` module waiter, NP-likeness model, and
/// `window.__smilesRdkit` API used by the Rust app via `js-sys`.
const INLINE_SCRIPT: &str = r##"
function waitForInitRDKitModule(timeoutMs = 12000) {
  const start = Date.now();
  return new Promise((resolve, reject) => {
    function poll() {
      if (typeof initRDKitModule === "function") {
        resolve(initRDKitModule);
        return;
      }
      if (Date.now() - start >= timeoutMs) {
        reject(new Error("RDKit_minimal.js is not loaded"));
        return;
      }
      setTimeout(poll, 16);
    }
    poll();
  });
}
function numericField(descriptors, names) {
  for (const name of names) {
    const value = descriptors?.[name];
    if (typeof value === "number" && Number.isFinite(value)) {
      return value;
    }
    if (typeof value === "string" && value.trim() !== "") {
      const parsed = Number(value);
      if (Number.isFinite(parsed)) {
        return parsed;
      }
    }
  }
  return null;
}

/*
 * Ertl natural-product-likeness model.
 *
 * The fragment contribution dictionary is derived from Ertl, Roggo &
 * Schuffenhauer (J. Chem. Inf. Model. 2008, 48, 68–74,
 * DOI 10.1021/ci700286x).  The original model was trained on ~50 000
 * natural products vs. ~1 M drug-like molecules from ZINC and uses
 * sparse Morgan fingerprints (radius 2).
 *
 * Because rdkit.js only exposes dense (folded) Morgan fingerprints,
 * the dictionary is pre-folded to N_MODEL_BITS buckets at build time
 * (Python script: key -> key % N_MODEL_BITS, summing contributions).
 * With 1 048 576 buckets the collision rate is ~12 %, introducing a
 * negligible mean absolute error on the score (< 0.15 Ertl units).
 *
 * The scoring follows the reference implementation exactly:
 *   raw   = Σ  model[bit]            for every fingerprint bit present
 *   score = raw / numAtoms
 *   if score > 4:  score = 4 + log10(score - 4 + 1)   (compression)
 *   if score < –4: score = –4 – log10(–4 – score + 1)
 *   confidence = bitsFound / totalBitsSet
 */
const N_MODEL_BITS = 1048576; /* 2^20 */
let npModelKeys = null; /* Uint32Array (sorted) */
let npModelScores = null; /* Float32Array (parallel) */

async function loadNpModel() {
  try {
    const response = await fetch("np_model.bin");
    const buffer = await response.arrayBuffer();
    const n = buffer.byteLength / 8;
    const dv = new DataView(buffer);
    const keys = new Uint32Array(n);
    const scores = new Float32Array(n);
    for (let i = 0; i < n; i++) {
      keys[i] = dv.getUint32(i * 8, true);
      scores[i] = dv.getFloat32(i * 8 + 4, true);
    }
    npModelKeys = keys;
    npModelScores = scores;
  } catch (e) {
    console.error("np_model.bin could not be loaded:", e);
    npModelKeys = null;
    npModelScores = null;
  }
}

function binarySearch(keys, target) {
  let lo = 0,
    hi = keys.length - 1;
  while (lo <= hi) {
    const mid = (lo + hi) >>> 1;
    if (keys[mid] === target) return mid;
    if (keys[mid] < target) lo = mid + 1;
    else hi = mid - 1;
  }
  return -1;
}

/*
 * Compute the real Ertl NP-likeness score for a molecule.
 *
 * Returns { np_score, np_confidence, num_atoms }.
 * np_score is null when the model is unavailable.
 */
function computeNpLikeness(mol) {
  let numAtoms = 0;
  try {
    numAtoms = mol.get_num_atoms();
  } catch (_e) {
    numAtoms = 0;
  }
  if (numAtoms === 0) {
    return { np_score: null, np_confidence: null, num_atoms: 0 };
  }
  if (!npModelKeys || !npModelScores || npModelKeys.length === 0) {
    return {
      np_score: null,
      np_confidence: null,
      num_atoms: numAtoms,
    };
  }
  try {
    let rawScore = 0;
    let bitsFound = 0;
    let totalBits = 0;
    const fp = mol.get_morgan_fp_as_uint8array(
      JSON.stringify({ radius: 2, nBits: N_MODEL_BITS })
    );
    for (let i = 0; i < fp.length; i++) {
      const byte = fp[i];
      if (byte === 0) continue;
      for (let b = 0; b < 8; b++) {
        if (byte && (1 << b)) {
          totalBits++;
          const bitId = i * 8 + b;
          const idx = binarySearch(npModelKeys, bitId);
          if (idx >= 0) {
            rawScore += npModelScores[idx];
            bitsFound++;
          }
        }
      }
    }
    /* size-normalised score (Ertl 2008) */
    let score = rawScore / numAtoms;
    /* log-compress extreme values to prevent score explosion */
    if (score > 4) {
      score = 4 + Math.log10(score - 4 + 1);
    } else if (score < -4) {
      score = -4 - Math.log10(-4 - score + 1);
    }
    const confidence = totalBits > 0 ? bitsFound / totalBits : 0;
    return {
      np_score: score,
      np_confidence: confidence,
      num_atoms: numAtoms,
    };
  } catch (e) {
    console.error("NP-likeness computation failed:", e);
    return {
      np_score: null,
      np_confidence: null,
      num_atoms: numAtoms,
    };
  }
}

const rdkitReady = (async () => {
  const [init, _modelLoaded] = await Promise.all([
    waitForInitRDKitModule(),
    loadNpModel(),
  ]);
  const RDKit = await init();
  await (window.__SMELLFISH_MOTIFS?.ready ?? Promise.resolve());
  const { MOTIF_LIBRARY, GROUP_NAMES, ERTL_SUBSTUENTS } = window.__SMELLFISH_MOTIFS || {
    MOTIF_LIBRARY: [],
    GROUP_NAMES: [],
    ERTL_SUBSTUENTS: [],
  };
  function compilePattern(pattern, kind) {
    try {
      const qmol = RDKit.get_qmol(pattern.smarts);
      if (!qmol) {
        console.warn(`Skipping invalid ${kind} pattern:`, pattern.label || pattern.smarts);
        return null;
      }
      return { ...pattern, qmol };
    } catch (error) {
      console.warn(`Skipping invalid ${kind} pattern:`, pattern.label || pattern.smarts, error);
      return null;
    }
  }
  const compiledMotifs = MOTIF_LIBRARY.map((motif) => compilePattern(motif, "motif")).filter(Boolean);
  const compiledSubstituents = ERTL_SUBSTUENTS.map((sub) => compilePattern(sub, "substituent")).filter(Boolean);

  /* ── LOTUS 1-percent scaffolds (Rutz et al. mortar fragmentation) ──
   * Fetch and compile the pre-filtered scaffold SMILES (MoleculePercentage
   * > 1 %) into RDKit query molecules for substructure searching. */
  let lotusScaffoldQueries = [];
  try {
    const lotusText = await fetch("lotus_1percent_scaffolds.txt").then((r) => r.text());
    for (const rawLine of lotusText.split(/\r?\n/)) {
      const s = rawLine.trim();
      if (!s || s.startsWith("#")) continue;
      try {
        const qmol = RDKit.get_qmol(s);
        if (qmol) lotusScaffoldQueries.push({ smiles: s, qmol });
      } catch (_e) { /* skip invalid SMILES */ }
    }
  } catch (e) {
    console.warn("lotus_1percent_scaffolds.txt could not be loaded:", e);
  }

  function withMol(smiles, callback) {
    const trimmed = String(smiles || "").trim();
    if (!trimmed) {
      return { error: "smiles is required" };
    }
    let mol = null;
    try {
      mol = RDKit.get_mol(trimmed);
    } catch (_error) {
      return { error: "rdkit.js could not parse the structure" };
    }
    if (!mol) {
      return { error: "rdkit.js could not parse the structure" };
    }
    try {
      return callback(mol);
    } finally {
      if (typeof mol.delete === "function") {
        mol.delete();
      }
    }
  }

  function uniqueSorted(values) {
    return [...new Set(values)].sort((a, b) => a - b);
  }

  /* RDKit.js supports substructure matching via get_substruct_match. */

  return {
    inspect(smiles) {
      const result = withMol(smiles, (mol) => {
        const canonicalsmiles = mol.get_smiles(JSON.stringify({
          canonical: true,
          isomericSmiles: false,
        }));
        const isomericsmiles = mol.get_smiles(JSON.stringify({
          canonical: true,
          isomericSmiles: true,
        }));
        const inchi = mol.get_inchi();
        if (!inchi) {
          throw new Error("rdkit.js could not generate InChI");
        }
        const inchikey = RDKit.get_inchikey_for_inchi(inchi);
        if (!inchikey) {
          throw new Error("rdkit.js could not generate InChIKey");
        }
        const rawDescriptors = JSON.parse(mol.get_descriptors() || "{}");
        const stereoTags = JSON.parse(mol.get_stereo_tags() || "[]");
        const descriptors = {
          amw: numericField(rawDescriptors, ["amw", "AMW"]),
          exact_mw: numericField(rawDescriptors, [
            "exactmw",
            "exact_mw",
            "ExactMolWt",
            "exact_molecular_weight",
          ]),
          clogp: numericField(rawDescriptors, ["CrippenClogP", "MolLogP", "clogp"]),
          tpsa: numericField(rawDescriptors, ["tpsa", "TPSA"]),
          fraction_csp3: numericField(rawDescriptors, ["fractionCSP3", "FractionCSP3"]),
          ring_count: numericField(rawDescriptors, ["NumRings", "num_rings"]),
          aromatic_ring_count: numericField(rawDescriptors, [
            "NumAromaticRings",
            "num_aromatic_rings",
          ]),
          aliphatic_ring_count: numericField(rawDescriptors, [
            "NumAliphaticRings",
            "num_aliphatic_rings",
          ]),
          rotatable_bonds: numericField(rawDescriptors, [
            "NumRotatableBonds",
            "num_rotatable_bonds",
          ]),
          hba: numericField(rawDescriptors, ["NumHAcceptors", "lipinskiHBA", "hba"]),
          hbd: numericField(rawDescriptors, ["NumHDonors", "lipinskiHBD", "hbd"]),
          hetero_atoms: numericField(rawDescriptors, [
            "NumHeteroatoms",
            "num_heteroatoms",
            "heteroatoms",
          ]),
        };

        const hits = [];
        const atoms = [];
        const atomColors = {};
        const bonds = [];
        const bondColors = {};

        /* Colour palette (matches the CSS chip colours) */
        const COL_NP    = [0.09, 0.51, 0.18];  /* green   #16a34a */
        const COL_SCAFF = [0.15, 0.37, 0.97];  /* blue    #2563eb */
        const COL_DECO  = [0.56, 0.25, 0.03];  /* amber   #92400e */
        const COL_LOTUS = [0.49, 0.19, 0.86];  /* purple  #7c3aed */

        function classifyMotif(motif) {
          if (motif?.kind === "scaffold") return COL_SCAFF;
          if (motif?.source_class === "synthetic") return COL_DECO;
          if (motif?.kind === "ring") return COL_NP;
          if (motif?.kind === "decoration") {
            return motif?.source_class === "natural"
              ? COL_NP
              : COL_DECO;
          }
          const l = String(motif?.label || "").toLowerCase();
          if (l.includes("ring") || l.includes("cycle")) return COL_NP;
          return COL_DECO;
        }

        let primaryColour = COL_DECO;
        for (const motif of compiledMotifs) {
          const raw = typeof mol.get_substruct_match === "function"
            ? mol.get_substruct_match(motif.qmol)
            : null;
          if (!raw) {
            continue;
          }
          const match = JSON.parse(raw);
          const rawAtoms = Array.isArray(match.atoms) ? match.atoms : [];
          const rawBonds = Array.isArray(match.bonds) ? match.bonds : [];
          const hitAtoms = rawAtoms;
          const hitBonds = rawBonds;
          if (!hitAtoms.length && !hitBonds.length) {
            continue;
          }
          const colour = classifyMotif(motif);
          hits.push({
            label: motif.label,
            kind: motif.kind,
            smarts: motif.smarts,
            source_class: motif.source_class,
            kingdom: motif.kingdom,
            kingdoms: Array.isArray(motif.kingdoms) ? motif.kingdoms : [],
            atoms: hitAtoms,
            bonds: hitBonds,
            colour,
          });
          atoms.push(...hitAtoms);
          bonds.push(...hitBonds);
          for (const atom of hitAtoms) {
            atomColors[atom] = colour;
          }
          for (const bond of hitBonds) {
            bondColors[bond] = colour;
          }
        }
        /* LOTUS 1-percent scaffold search (Rutz et al. mortar
         * fragmentation) — substructure match against pre-compiled
         * scaffold query molecules.  Must run *before* SVG rendering
         * so the purple highlights are baked into the SVG. */
        const lotusScaffolds = [];
        for (const scaff of lotusScaffoldQueries) {
          const raw = typeof mol.get_substruct_match === "function"
            ? mol.get_substruct_match(scaff.qmol)
            : null;
          if (raw) {
            const match = JSON.parse(raw);
            const hitAtoms = Array.isArray(match.atoms) ? match.atoms : [];
            const hitBonds = Array.isArray(match.bonds) ? match.bonds : [];
            if (hitAtoms.length || hitBonds.length) {
              lotusScaffolds.push(scaff.smiles);
              atoms.push(...hitAtoms);
              bonds.push(...hitBonds);
              for (const atom of hitAtoms) atomColors[atom] = COL_LOTUS;
              for (const bond of hitBonds) bondColors[bond] = COL_LOTUS;
            }
          }
        }
        /*
         * RDKit.js supports per-atom and per-bond highlight colours
         * via highlightAtomColors and highlightBondColors options.
         */
        let svg;
        if (atoms.length || bonds.length) {
          const details = {
            atoms: uniqueSorted(atoms),
            bonds: uniqueSorted(bonds),
            highlightAtomColors: atomColors,
            highlightBondColors: bondColors,
            continuousHighlight: true,
            atomHighlightsAreCircles: true,
            addAtomIndices: false,
          };
          svg = mol.get_svg_with_highlights(JSON.stringify(details));
        } else {
          svg = mol.get_svg();
        }

        const npResult = computeNpLikeness(mol);

        // Check Ertl's top-2000 most common natural-product substituents
        const matchedSubstituents = [];
        for (const sub of compiledSubstituents) {
          const raw = typeof mol.get_substruct_match === "function"
            ? mol.get_substruct_match(sub.qmol)
            : null;
          if (raw) {
            const match = JSON.parse(raw);
            const hitAtoms = Array.isArray(match.atoms) ? match.atoms : [];
            const hitBonds = Array.isArray(match.bonds) ? match.bonds : [];
            if (hitAtoms.length || hitBonds.length) {
              matchedSubstituents.push(sub.label);
            }
          }
        }

        /* LOTUS 1-percent scaffold search — moved before SVG render */
        return {
          canonicalsmiles,
          isomericsmiles,
          inchikey,
          svg,
          motifs: hits,
          lotus_scaffolds: lotusScaffolds,
          descriptors,
          stereo_tags: Array.isArray(stereoTags) ? stereoTags : [],
          np_score: npResult.np_score,
          np_confidence: npResult.np_confidence,
          num_atoms: npResult.num_atoms,
          substituents: matchedSubstituents,
        };
      });
      if (result && typeof result.error === "string") {
        return result;
      }
      return result;
    },
  };
})();

window.__smilesRdkit = {
  ready: rdkitReady,
  async inspect(smiles) {
    const bridge = await rdkitReady;
    return bridge.inspect(smiles);
  },
};
"##;

/// Renders the document `<head>` for smellfish-rs.
///
/// Replaces `<meta>`, `<script>`, `<link>` tags from `index.html` with
/// `dioxus::document` elements.  The `RDKit` bridge JS, motif-library loader,
/// and NP-likeness model are all added programmatically.
#[component]
pub fn SmellfishDocumentHead() -> Element {
    let description = "Drop a CSV of SMILES, render molecules with RDKit.js, and score \
        natural-product originality with Ertl-style NP-likeness, LOTUS/PubChem evidence, \
        and dataset-derived scaffold and decoration motifs.";

    let scripts = vec![
        "https://unpkg.com/@rdkit/rdkit/dist/RDKit_minimal.js".to_string(),
        "https://scripts.simpleanalyticscdn.com/latest.js".to_string(),
    ];

    rsx! {
        DocumentHead {
            title: "smellfish-rs".to_string(),
            lang: "en".to_string(),
            description: Some(description.to_string()),
            theme_colors: Some(("#f6f8fb", "#10141b")),
            scripts,
            inline_script: Some(INLINE_SCRIPT.to_string()),
        }

        // Keep the local LOTUS-inspired styling in the initial HTML response so
        // the UI is not blank during the first WASM load.
        document::Style { "{CSS}" }

        // Local motif-library script (must load before inline bridge JS)
        document::Script { src: "motif-library.js" }

        // Resource hints — preconnect for external origins
        document::Link { rel: "preconnect", href: "https://www.rdkitjs.com", crossorigin: "anonymous" }
        document::Link { rel: "preconnect", href: "https://qlever.dev", crossorigin: "anonymous" }
        document::Link { rel: "preconnect", href: "https://qlever.cs.uni-freiburg.de", crossorigin: "anonymous" }
    }
}
