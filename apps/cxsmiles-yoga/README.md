# 🧘 CX-SMILES Yoga

Generate a single **CX-SMILES** from a list of related structures — and validate it
back by re-expansion.

A CX-SMILES encodes *structural uncertainty*: a positional isomer (`m:` block) or a
variable-length repeat (`Sg:n:` block). Hand-crafting these does not scale, so this
app derives them automatically from a candidate structure list using the **maximum
common substructure** as the shared scaffold.

> CXSMILES-Yoga working session — 28 April 2026. Attendees: Robin Schmid,
> Carolin Huber, Florian Huber, Adriano Rutz, Filip Jozefov, Justin van der Hooft.
> (Session notes: internal; no public URL.)

---

## How it works

The UI (`src/app.rs`) is intentionally thin. All chemistry lives in a headless,
unit-tested module (`src/cxsmiles.rs`) so it can be reused outside the browser.

```
 input SMILES list   ──▶  parse (chematic::smiles)
                        ──▶  cluster (ECFP4 + Tanimoto, τ = 0.3)
                              └─ rejects unrelated structures so they are not
                                 forced into one nonsensical CX-SMILES
                        ──▶  Maximum Common Substructure (chematic::smarts::find_mcs)
                        ──▶  diff group vs. MCS → classify each variable region
                              • discrete equivalents  →  m:   block
                              • variable-length repeat →  Sg:n: block
                        ──▶  serialize to a CX-SMILES string
                              (m: /Sg:n: are hand-rolled — chematic's CX reader
                               does not support these constructs)
                        ──▶  round-trip: enumerate the CX-SMILES and check that
                              it re-expands to (a superset of) the inputs
                              └─ surfaced as a confidence % in the UI
```

### Construct families

| Family | CX-SMILES construct | Example |
|---|---|---|
| Positional isomers | `m:<starIdx>:<pos1>.<pos2>.…` | `…Cl* \|m:13:0.2.3\|` for Cl on a biphenyl at three positions |
| Variable-length repeat | `Sg:n:<atomRange>:n:ht` | `OC(=O)C(F)(F)C(F)F \|Sg:n:3,4,5:n:ht\|` (PFAS) / `CCCCCCC \|Sg:n:3:n:ht\|` (alkyl chain) |
| Best-effort (no clean scaffold) | `m:` from the MCS, flagged lower-confidence | the six PFOS constitutional isomers |

### Confidence

The round-trip stage enumerates the generated CX-SMILES back into concrete
structures and counts how many of the original inputs are recovered
(**coverage = covered / total**). When every input re-expands, confidence is
*clean* (100%); otherwise the construct is flagged as lower-confidence — this is
how the "no clean shared scaffold" constitutional-isomer case is surfaced.

### 2D depiction is fully local

All molecule rendering uses `chematic::depict` (`depict_svg` /
`depict_svg_highlighted`, which run an in-process 2D layout) — no RDKit.js,
no remote service, no network calls. The scaffold and each enumerated candidate
are drawn inline in the results panel.

---

## Try it

```bash
# Run the test suite (covers all six fixtures + a parse/write round-trip)
cargo test -p cxsmiles-yoga --lib
```

The headless core is exercised by seven unit tests in `src/cxsmiles.rs` covering the
six seed fixtures — biphenyl-Cl, moving -OCH₃, the double-`m:` acetyl case, the PFAS
repeat, the alkyl repeat, and the six constitutional PFOS isomers — plus a
parse/write round-trip.

### Run in the browser

```bash
cargo build --release --target wasm32-unknown-unknown -p cxsmiles-yoga
# then `dx serve -p cxsmiles-yoga` or serve the .wasm with Dioxus SSR/wasm
```

---

## Limitations & out of scope

- **Aromatic equivalence is NYI.** The "enumerate equivalent aromatic positions"
  checkbox is wired but not yet implemented — CX-SMILES aromatic `m:` enumeration
  for equivalent ring positions is left for a follow-up.
- **CX-SMILES hand-rolled.** `chematic::cx` (`CxSmiles` / `parse_cxsmiles` /
  `write_cxsmiles`) does not support `m:` / `Sg:n:` constructs, so the
  atom-index logic is implemented directly (see the *Findings* note in the
  session summary: chematic's SMILES writer also never emits `*` for wildcard
  atoms, so the base scaffold is assembled as a string, not via `write`).
- **Hashing / fingerprint library search** — comparing a generated CX-SMILES
  against a fingerprint library (e.g. ECFP4/Tanimoto nearest-neighbour lookup)
  to find the best representative or to score candidates is out of scope for
  this app. Use the `crates/lotus` / `crates/upload` plumbing + `chematic::fp`
  elsewhere in the workspace for that.

---

## Dependencies

- [`chematic`](https://crates.io/crates/chematic) `0.4.27` (workspace-resolved to `0.4.30`)
  — SMILES parsing/writing, SMARTS/MCS (`find_mcs`, `find_matches`), ECFP4
  fingerprints, 2D depiction.
- [`dioxus`](https://dioxuslabs.com) `0.7` — UI.
- `web-sys` — clipboard API (`Window` + `Navigator` + `Clipboard` features).
- `gloo-timers` (workspace) — async clipboard "Copied!" flash.
- [`crates/ui`](../crates/ui) + [`crates/upload`](../crates/upload) — shared
  components (`Header` / `Footer` / `Card` / `NoticeBar` / `SegmentedControl`
  / `DocumentHead`) and the file-upload plumbing, reused from `lipid-selecto-rs`.
- [`rustc-hash`](https://crates.io/crates/rustc-hash) — `FxHashMap` for match
  maps (`type Match = rustc_hash::FxHashMap<usize, AtomIdx>`).
