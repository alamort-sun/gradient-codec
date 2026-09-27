# Hubble Mirror — Rev 2

**Status:** Ratified with amendments A–C.

## Standing direction

Observation semantics expand **around** canonical `Vector15D`; nothing enters or replaces the 15-field geometry. The envelope is context and provenance, never a second geometry. Predictions, integrations, renderings, and posterior estimates remain derived products subject to the pinned codec.

## Adopted amendments

### A. Derived renderer defect

`hue()` returns eight labels, including the out-of-range `white` fallback. Passing those labels into `hsl(...)` produced invalid CSS for every non-void glyph. P0 now emits numeric `su2_polarity` as the HSL angle and treats `hue()` labels as descriptive output only.

### B. Corpus is the gate

The current evidence is 85 segments from one source, reduced to two aggregate anchors. It cannot establish population validity, robust region boundaries, calibrated uncertainty, or posterior inference.

Sequence:

1. P0 lands now.
2. Corpus collection starts and runs continuously.
3. Observation-envelope and uncertainty design may proceed, but scientific claims remain hypotheses.
4. Integration experiments require independent observations.
5. Transform graphs, lens operators, and posterior inference reopen on corpus readiness—not calendar milestones.

### C. Covariance-ready uncertainty

`StateUncertainty` begins with per-field sigma but reserves an optional sparse-covariance block in the first schema. Sparse entries use stable continuous-field identifiers and a declared basis/version. Missing covariance means “not supplied,” never independence or zero covariance.

This belongs to the observation envelope, not `Vector15D`.

## Corrections

- `hue()` returns eight strings, not seven: `white`, `red`, `orange`, `yellow`, `green`, `blue`, `purple`, and `pink`.
- Pole zero-default behavior is already declared in `INVARIANTS.md`; it is compatibility debt, not newly discovered drift.
- Canonical typed geometry is `R^13 × DomainWall × GaugeCoupling`: 13 continuous fields, including both poles, plus two categorical fields.

## Corpus gate

“Corpus ready” requires a ratified protocol, not an arbitrary sample count. At minimum it must include:

- A second independent source.
- Repeated sessions per source.
- Capture and calibration metadata.
- Raw or loss-accounted source retention.
- Held-out sessions not used to define region boundaries.
- Explicit consent, retention, and deletion policy.
- Enough observations to estimate repeatability and field coupling without treating segments from one recording as independent participants.

Until then, region usefulness, cross-field coupling, population generalization, and inverse inference stay `HYPOTHESIS`.

## Phase authority

| Work | Status | Gate |
|---|---|---|
| P0: notation, validation layers, contextual detection, HSL repair | Approved now | Existing verified repository law |
| Envelope, evidence classes, uncertainty schema | Design-approved | Must remain outside geometry |
| Integration engine | Design-approved, scientific use gated | Independent corpus observations |
| Spectral channels | Direction-approved | Corpus plus extraction/calibration contract |
| Transform graph, lens operators, posterior inference | Direction-approved only | Corpus readiness and independent validation |
