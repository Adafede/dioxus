window.__SMELLFISH_MOTIFS = {
  MOTIF_LIBRARY: [],
  USER_SCAFFOLDS: [],
  ERTL_SUBSTUENTS: [],
  ready: (async () => {
    const [sourceText, kingdomText, scaffoldText, substText] = await Promise.all([
      fetch("ertl_source_vs_synthetic.txt").then((r) => r.text()),
      fetch("ertl_kingdom_enrichment.txt").then((r) => r.text()),
      fetch("user_scaffolds.txt").then((r) => r.text()),
      fetch("ertl_npsubstituents.txt").then((r) => r.text()),
    ]);

    const sourceRows = parseTable(sourceText);
    const kingdomRows = parseTable(kingdomText);
    const scaffoldRows = parsePatternList(scaffoldText);
    const motifs = buildMotifLibrary(sourceRows, kingdomRows, scaffoldRows);
    const substituents = parseSubstituents(substText);

    window.__SMELLFISH_MOTIFS.MOTIF_LIBRARY = motifs;
    window.__SMELLFISH_MOTIFS.USER_SCAFFOLDS = scaffoldRows;
    window.__SMELLFISH_MOTIFS.ERTL_SUBSTUENTS = substituents;
  })(),
};

function parseTable(text) {
  const rows = [];
  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#")) continue;
    const parts = line.split(/\s+/);
    const label = parts.shift();
    if (!label || label === "FG") continue;
    const values = parts.map((part) => Number(part)).filter((n) => Number.isFinite(n));
    if (!values.length) continue;
    rows.push({ label, values });
  }
  return rows;
}

function buildMotifLibrary(sourceRows, kingdomRows, scaffoldRows) {
  const sourceMap = new Map(sourceRows.map((row) => [row.label, row]));
  const kingdomMap = new Map(kingdomRows.map((row) => [row.label, row]));
  const scaffoldLabels = new Set(scaffoldRows.map((row) => row.label));
  const labels = new Set([...sourceMap.keys(), ...kingdomMap.keys()]);
  const motifs = [];

  for (const label of labels) {
    const source = sourceMap.get(label);
    const kingdom = kingdomMap.get(label);
    const sourceSplit = source ? classifySource(source.values) : null;
    const kingdomSplit = kingdom ? classifyKingdom(kingdom.values) : null;

    motifs.push({
      label,
      kind: classifyKind(label, sourceSplit, scaffoldLabels),
      smarts: labelToSmarts(label),
      source_class: sourceSplit?.label || "unknown",
      kingdom: kingdomSplit?.label || "unknown",
      kingdoms: kingdomSplit?.kingdoms || [],
      source_score: sourceSplit?.delta ?? 0,
      kingdom_score: kingdomSplit?.score ?? 0,
    });
  }

  motifs.sort((left, right) => {
    return (right.source_score || 0) - (left.source_score || 0) ||
      (right.kingdom_score || 0) - (left.kingdom_score || 0) ||
      left.kind.localeCompare(right.kind) ||
      left.label.localeCompare(right.label);
  });
  return motifs;
}

function classifySource(values) {
  const synthetic = values[values.length - 1] ?? 0;
  const np = Math.max(...values.slice(0, -1), 0);
  return {
    label: np >= synthetic ? "natural" : "synthetic",
    delta: np - synthetic,
  };
}

function classifyKingdom(values) {
  const labels = ["animals", "plants", "fungi", "bacteria", "synthetic"];
  const synthetic = values[values.length - 1] ?? 0;
  const enriched = [];
  let bestDelta = Number.NEGATIVE_INFINITY;

  for (let i = 0; i < labels.length - 1; i++) {
    const value = values[i] ?? 0;
    const delta = value - synthetic;
    if (delta > bestDelta) {
      bestDelta = delta;
    }
    if (value >= 5 && delta >= 5 && (synthetic === 0 || value >= synthetic * 1.5)) {
      enriched.push(labels[i]);
    }
  }

  if (!enriched.length) {
    return { label: "unknown", kingdoms: [], score: bestDelta };
  }
  if (enriched.length === 1) {
    return { label: enriched[0], kingdoms: enriched, score: bestDelta };
  }
  return { label: "multiple kingdoms", kingdoms: enriched, score: bestDelta };
}

function classifyKind(label, sourceSplit, scaffoldLabels) {
  const l = label.toLowerCase();
  if (scaffoldLabels.has(label)) return "scaffold";
  if (sourceSplit?.label === "synthetic") return "decoration";
  if (
    l.includes("ring") ||
    l.includes("cycle") ||
    l.includes("macro") ||
    l.includes("steroid") ||
    l.includes("sugar") ||
    l.includes("flav") ||
    l.includes("indole") ||
    l.includes("quin") ||
    l.includes("pyr") ||
    l.includes("furan") ||
    l.includes("thiophene")
  ) {
    return "ring";
  }
  return "decoration";
}

function parsePatternList(text) {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("#"))
    .map((line) => {
      const parts = line.split(/\s+/);
      const label = parts.shift();
      if (!label) return null;
      return {
        label,
        smarts: parts.length ? parts.join(" ") : label,
      };
    })
    .filter(Boolean);
}

function labelToSmarts(label) {
  return label
    .replaceAll("[R]", "[*]")
    .replaceAll("[Oar+]", "[o+]")
    .replaceAll("[Nar+]", "[n+]")
    .replaceAll("[Oar]", "o")
    .replaceAll("[Nar]", "n")
    .replaceAll("[Sar]", "s")
    .replaceAll("[Car]", "c")
    .replaceAll("[Cal]", "C");
}

function parseSubstituents(text) {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("#"))
    .map((line) => ({
      label: line,
      smarts: line.replaceAll("[R]", "[*]"),
    }));
}
