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
function waitForSmellfishMotifs(timeoutMs = 5000) {
  const start = Date.now();
  return new Promise((resolve) => {
    function poll() {
      if (window.__SMELLFISH_MOTIFS) {
        resolve(window.__SMELLFISH_MOTIFS);
        return;
      }
      if (Date.now() - start >= timeoutMs) {
        console.warn("motif-library.js did not initialize before timeout");
        resolve(null);
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
 * DOI 10.1021/ci700286x) using sparse Morgan fingerprints (radius 2).
 *
 * Because RDKit_minimal.js exposes only binary folded Morgan bit vectors
 * (`get_morgan_fp_as_uint8array`) rather than unfolded count fingerprints,
 * `np_model.bin` pre-folds fragment keys modulo N_MODEL_BITS by summing
 * colliding contributions. This provides an empirical folded approximation
 * rather than exact bit-count Ertl scoring.
 */
const N_MODEL_BITS = 1048576; /* 2^20 */
const FP_OPTS = JSON.stringify({ radius: 2, nBits: N_MODEL_BITS });
const CANONICAL_SMILES_OPTS = JSON.stringify({
  canonical: true,
  isomericSmiles: false,
});
const ISOMERIC_SMILES_OPTS = JSON.stringify({
  canonical: true,
  isomericSmiles: true,
});
let npModelKeys = null; /* Uint32Array (sorted) */
let npModelScores = null; /* Float32Array (parallel) */

async function loadNpModel() {
  try {
    const response = await fetch("np_model.bin");
    if (!response.ok) {
      throw new Error(`HTTP ${response.status} loading np_model.bin`);
    }
    const buffer = await response.arrayBuffer();
    if (buffer.byteLength === 0 || buffer.byteLength % 8 !== 0) {
      throw new Error(`Invalid np_model.bin byteLength: ${buffer.byteLength}`);
    }
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
function computeNpLikeness(mol, heavyAtoms) {
  let numAtoms =
    typeof heavyAtoms === "number" && Number.isFinite(heavyAtoms) && heavyAtoms > 0
      ? heavyAtoms
      : 0;
  if (numAtoms === 0) {
    try {
      numAtoms = mol.get_num_atoms();
    } catch (_e) {
      numAtoms = 0;
    }
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
    const fp = mol.get_morgan_fp_as_uint8array(FP_OPTS);
    for (let i = 0; i < fp.length; i++) {
      const byte = fp[i];
      if (byte === 0) continue;
      for (let b = 0; b < 8; b++) {
        if ((byte & (1 << b)) !== 0) {
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
  const motifsObj = await waitForSmellfishMotifs();
  await (motifsObj?.ready ?? Promise.resolve());
  const { MOTIF_LIBRARY, ERTL_SUBSTITUENTS } = motifsObj || {
    MOTIF_LIBRARY: [],
    ERTL_SUBSTITUENTS: [],
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
  const compiledSubstituents = ERTL_SUBSTITUENTS.map((sub) => compilePattern(sub, "substituent")).filter(Boolean);

  /* ── LOTUS 1-percent scaffolds (Rutz et al. mortar fragmentation) ──
   * Fetch and compile the pre-filtered scaffold SMILES (MoleculePercentage
   * > 1 %) into RDKit query molecules for substructure searching. */
  let lotusScaffoldQueries = [];
  try {
    const lotusText = await fetch("lotus_1percent_scaffolds.txt").then((r) => {
      if (!r.ok) {
        throw new Error(`HTTP ${r.status} loading lotus_1percent_scaffolds.txt`);
      }
      return r.text();
    });
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

  function extractSubstructMatches(mol, qmol) {
    if (typeof mol.get_substruct_matches === "function") {
      const raw = mol.get_substruct_matches(qmol);
      if (raw) {
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed)) {
          return parsed;
        }
      }
    }
    if (typeof mol.get_substruct_match === "function") {
      const raw = mol.get_substruct_match(qmol);
      if (raw) {
        const match = JSON.parse(raw);
        if (match && (Array.isArray(match.atoms) || Array.isArray(match.bonds))) {
          return [match];
        }
      }
    }
    return [];
  }

  return {
    inspect(smiles) {
      return withMol(smiles, (mol) => {
        const canonicalsmiles = mol.get_smiles(CANONICAL_SMILES_OPTS);
        const isomericsmiles = mol.get_smiles(ISOMERIC_SMILES_OPTS);
        const inchi = mol.get_inchi();
        if (!inchi) {
          throw new Error("rdkit.js could not generate InChI");
        }
        const inchikey = RDKit.get_inchikey_for_inchi(inchi);
        if (!inchikey) {
          throw new Error("rdkit.js could not generate InChIKey");
        }
        const rawDescriptors = JSON.parse(mol.get_descriptors() || "{}");
        const parsedStereo = JSON.parse(mol.get_stereo_tags() || "{}");
        const stereoTags = Array.isArray(parsedStereo)
          ? parsedStereo.map((t) => (Array.isArray(t) ? t.join(":") : String(t)))
          : [
              ...(Array.isArray(parsedStereo?.CIP_atoms)
                ? parsedStereo.CIP_atoms.map((t) =>
                    Array.isArray(t) ? t.join(":") : String(t),
                  )
                : []),
              ...(Array.isArray(parsedStereo?.CIP_bonds)
                ? parsedStereo.CIP_bonds.map((t) =>
                    Array.isArray(t) ? t.join(":") : String(t),
                  )
                : []),
            ];
        const heavyAtoms = numericField(rawDescriptors, [
          "NumHeavyAtoms",
          "num_heavy_atoms",
          "numHeavyAtoms",
        ]);
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
        const atomPriority = {};
        const bonds = [];
        const bondColors = {};
        const bondPriority = {};

        /* Colour palette and priority hierarchy (COL_LOTUS > COL_SCAFF > COL_NP > COL_DECO) */
        const COL_NP    = [0.09, 0.51, 0.18];  /* green   #16a34a */
        const COL_SCAFF = [0.15, 0.37, 0.97];  /* blue    #2563eb */
        const COL_DECO  = [0.56, 0.25, 0.03];  /* amber   #92400e */
        const COL_LOTUS = [0.49, 0.19, 0.86];  /* purple  #7c3aed */

        function classifyMotif(motif) {
          if (motif?.kind === "scaffold") return { colour: COL_SCAFF, priority: 3 };
          if (motif?.source_class === "synthetic") return { colour: COL_DECO, priority: 1 };
          if (motif?.kind === "ring") return { colour: COL_NP, priority: 2 };
          if (motif?.kind === "decoration") {
            return motif?.source_class === "natural"
              ? { colour: COL_NP, priority: 2 }
              : { colour: COL_DECO, priority: 1 };
          }
          const l = String(motif?.label || "").toLowerCase();
          if (l.includes("ring") || l.includes("cycle")) {
            return { colour: COL_NP, priority: 2 };
          }
          return { colour: COL_DECO, priority: 1 };
        }

        function assignHighlight(hitAtoms, hitBonds, colour, priority) {
          for (const atom of hitAtoms) {
            if (priority >= (atomPriority[atom] ?? 0)) {
              atomColors[atom] = colour;
              atomPriority[atom] = priority;
            }
          }
          for (const bond of hitBonds) {
            if (priority >= (bondPriority[bond] ?? 0)) {
              bondColors[bond] = colour;
              bondPriority[bond] = priority;
            }
          }
        }

        function extractMatchedIndices(matches) {
          return {
            hitAtoms: uniqueSorted(
              matches.flatMap((m) => (Array.isArray(m.atoms) ? m.atoms : [])),
            ),
            hitBonds: uniqueSorted(
              matches.flatMap((m) => (Array.isArray(m.bonds) ? m.bonds : [])),
            ),
          };
        }

        for (const motif of compiledMotifs) {
          const matches = extractSubstructMatches(mol, motif.qmol);
          if (!matches.length) continue;
          const { hitAtoms, hitBonds } = extractMatchedIndices(matches);
          if (!hitAtoms.length && !hitBonds.length) {
            continue;
          }
          const { colour, priority } = classifyMotif(motif);
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
          assignHighlight(hitAtoms, hitBonds, colour, priority);
        }
        /* LOTUS 1-percent scaffold search (Rutz et al. mortar
         * fragmentation) — substructure match against pre-compiled
         * scaffold query molecules.  Must run *before* SVG rendering
         * so the purple highlights are baked into the SVG. */
        const lotusScaffolds = [];
        for (const scaff of lotusScaffoldQueries) {
          const matches = extractSubstructMatches(mol, scaff.qmol);
          if (!matches.length) continue;
          const { hitAtoms, hitBonds } = extractMatchedIndices(matches);
          if (hitAtoms.length || hitBonds.length) {
            lotusScaffolds.push(scaff.smiles);
            atoms.push(...hitAtoms);
            bonds.push(...hitBonds);
            assignHighlight(hitAtoms, hitBonds, COL_LOTUS, 4);
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

        const npResult = computeNpLikeness(mol, heavyAtoms);

        // Check Ertl's top-2000 most common natural-product substituents
        const matchedSubstituents = [];
        for (const sub of compiledSubstituents) {
          const matches = extractSubstructMatches(mol, sub.qmol);
          for (const match of matches) {
            const hitAtoms = Array.isArray(match.atoms) ? match.atoms : [];
            const hitBonds = Array.isArray(match.bonds) ? match.bonds : [];
            if (hitAtoms.length || hitBonds.length) {
              matchedSubstituents.push(sub.label);
            }
          }
        }

        return {
          canonicalsmiles,
          isomericsmiles,
          inchikey,
          svg,
          motifs: hits,
          lotus_scaffolds: lotusScaffolds,
          descriptors,
          stereo_tags: stereoTags,
          np_score: npResult.np_score,
          np_confidence: npResult.np_confidence,
          num_atoms: npResult.num_atoms,
          substituents: matchedSubstituents,
        };
      });
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
