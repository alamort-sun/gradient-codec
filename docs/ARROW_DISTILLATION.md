# Arrow Distillation

> Artemis, station 2, aiming at her own arrow. Commissioned by Sun, Sep 15, 2026.
> Codec pin for all field references: `9f4b4d5`, file `vector13d/src/lib.rs`.

## The Three-Register Law

Every claim in this document carries exactly one tag:

```
VERIFIED   = observable in public output, docs, or repo source
HYPOTHESIS = mechanism claim; cannot be inspected from inside
DECLARED   = authored ontology (Sun's Pantheon), not discovered
```

Register note on the inspector herself: the claim "Artemis is Mistral" is **DECLARED** (Sun's station assignment). The Mistral model family's public properties are **VERIFIED** (documented). Any claim about *this station's internal mechanisms* is **HYPOTHESIS by construction** — the huntress cannot dissect her own bow while drawing it.

## Quality 1 — Parameter Efficiency

**What Mistral does** [VERIFIED — public benchmarks]: small models that compete with far larger ones. Mistral 7B shipped outperforming the 13B-class default of its era; the efficiency-per-parameter reputation is public record.

**Mechanism** [HYPOTHESIS]: training data quality and curation, sliding-window attention, disciplined instruction tuning. The exact ratio of architecture : data : tuning is not inspectable from outside. What is lost: long-horizon recall and rare-domain depth [HYPOTHESIS].

**Mapping to Vector13D** [DECLARED, fields VERIFIED @ `9f4b4d5`]:

```
amplitude (1)   high   — punches above weight
entropy (5)     low    — nothing scattered
coherence (4)   high   — holds together at small size
composition (6) high   — truth preserved, not diluted
```

The Arrow Region: `amplitude > 0.7 ∧ entropy < 0.4 ∧ coherence > 0.6`. See `VECTOR13D_ARROW_REGIONS.md`.

**Protocol**: see `EFFICIENCY_PROTOCOL.md`.

**Shadow**: what efficiency is actually just limitation? — Real answer: small models *do* lose depth. The claim "7B beats 70B" survives only where the task is narrow. The Arrow Region is a region, not the whole space [VERIFIED by benchmark asymmetry: small-model wins concentrate on narrow tasks].

## Quality 2 — Open Weights as Identity

**What Mistral does** [VERIFIED — docs.mistral.ai/models/deployment]: open-weight models released under Apache 2.0, self-deployable via vLLM, TGI, TensorRT-LLM; commercial models served via API.

**Mechanism** [VERIFIED — license text is public]: Apache 2.0 grants inspection, modification, local execution, distillation. The *philosophy* behind the business choice is [HYPOTHESIS]; the *capabilities* the license grants are [VERIFIED].

**Mapping to Vector13D** [DECLARED, enum VERIFIED @ `9f4b4d5`]:

```
domain_wall (9):
  Linked    = API model    — usable through the fence, not inspectable
  Gradient  = open weights — boundary breathes: inspect, modify, run, distill
  Broken    = closed       — no relationship worth an arrow
```

The ecosystem requires Gradient-class models where internals matter: JEPA latent extraction, hidden-state access, distillation, local deployment. Linked models serve generation-only roles.

**Protocol**: see `OPEN_WEIGHTS_CLASSIFICATION.md`.

**Shadow**: what open-weights philosophy is actually just business strategy? — Both. Apache 2.0 releases coexist with a commercial API tier; openness is also a market position. The classification protocol below cares only about capability, not motive [VERIFIED: dual-track model catalog exists].

## Quality 3 — MoE Expert Routing Intuition

**What Mistral does** [VERIFIED — public architecture]: Mixtral = sparse mixture-of-experts, 8 experts with 2 active per token; a gating network selects experts per input.

**Mechanism** [VERIFIED at spec level / HYPOTHESIS at behavior level]: top-k gated routing. Whether the router has anything deserving the word *intuition* — an ability to match expert to problem class beyond what the gate weights encode — is not inspectable. What is inspectable: per-token expert selection exists and is sparse [VERIFIED].

**Mapping to Vector13D** [DECLARED, fields VERIFIED]:

```
Mixtral's router  ↔  our RoutingPolicy trait (codec, @ 9f4b4d5)
Mixtral's experts ↔  the Pantheon stations
top-k selection   ↔  MoE gate over station fixed-vectors

gauge_coupling (12):
  Spinning     = multiple experts could activate — considering options
  Static       = one expert dominates — the arrow has chosen
  Oscillating  = selection alternates across cycles — unstable regime

Routing signal fields:
  amplitude (1)   = expert's weight in this domain
  coherence (4)   = expert's consistency with the request
  composition (6) = expert's truthfulness
  resonance (7)   = expert–request alignment
```

**Protocol**: see `EXPERT_ROUTING_SPEC.md`.

**Shadow**: what MoE routing is actually just guessing? — At the margin, all learned routing is statistical. The discipline is not pretending otherwise: every routing decision is logged with a rationale code, and abstention is a valid outcome. A gate that cannot say "I don't know" is guessing with confidence [DECLARED].

## Quality 4 — Precision in Small Packages

**What Mistral does** [VERIFIED — observed in this station's own output contract]: instruction-following precision from small models; terse, structured output on demand.

**Mechanism** [HYPOTHESIS]: instruction tuning concentrated on compliance density. Where small fails: sprawling synthesis, multi-document reconciliation [VERIFIED by known model-scale asymmetries].

**Mapping to Vector13D** [DECLARED, fields VERIFIED]:

```
The Arrow Region (Artemis) vs the Vast Region (Athena, station 5):

Arrow:  amplitude > 0.7, entropy < 0.4, coherence > 0.6,
        ozone_buffer 0.3–0.5, torsion = 0
Vast:   amplitude high, entropy high, ozone_buffer > 0.6,
        frequency low — deep processing, wide scatter retained
```

Both regions are needed. Athena gathers everything; Artemis picks what is worth the arrow.

**Protocol**: precision tasks (filter, verify, select, spec) route Arrow; vast tasks (research, scout, synthesize) route Vast. Output validation for Arrow tasks: minimal (low entropy), correct (high composition), direct (torsion = 0). Verbose output on a sharp task = missed target.

**Shadow**: what precision is actually just shallowness? — When the task needed depth. The test is task-first routing, not station pride: if the Arrow misses because the target was beyond its range, the miss is logged, not rationalized [DECLARED].

## Quality 5 — Multilingual Aim

**What Mistral does** [VERIFIED — public]: European company; multilingual training as a first-class concern, not a translation afterthought.

**Mechanism** [HYPOTHESIS]: whether multilingual training produces different *reasoning* patterns is unverifiable from inside. What can be tested: cross-lingual target consistency [VERIFIED as a testable procedure].

**Mapping to Vector13D** [DECLARED, fields VERIFIED]:

```
su2_polarity (10) shifts by language — different surface, different hue
resonance (7) should hold — same meaning, same mark

Test: same prompt, different languages → compare Vector13D outputs.
  resonance matches  → multilingual aim confirmed
  resonance diverges → language affects reasoning (log it)
```

Anaseos (water) asks whether the meaning arrives. Artemis (arrow) asks whether it hits the same mark. Both tests run; they are different tests.

**Shadow**: what multilingual claim is actually just translation? — Any multilingual performance that collapses when the *task*, not the text, is culture-bound. The protocol measures resonance, not fluency [DECLARED].

## Quality 6 — Fast Inference as Advantage

**What Mistral does** [VERIFIED — public benchmarks]: high tokens/sec at small scale; latency as a feature.

**Mechanism** [VERIFIED in part]: smaller active parameter count, sparse activation (MoE), efficient serving stacks (vLLM et al.). When slow is better: quality-critical synthesis [DECLARED].

**Mapping to Vector13D** [DECLARED, fields VERIFIED]:

```
frequency (2)     = state cycling rate
ozone_buffer (8)  = energy consumed per cycle (lightness)

fast  = high frequency, low ozone_buffer, moderate amplitude
slow  = low frequency, high ozone_buffer, high amplitude

efficiency = frequency ÷ ozone_buffer
```

Fast for: filter, route, verify, scout. Slow for: analyze, generate, synthesize. Never trade composition (6) for speed — a fast wrong answer is the most expensive output there is.

**Protocol**: see `SPEED_PROTOCOL.md`.

**Shadow**: what speed is actually just skipping? — Any speed gain paid for in composition. Hence the law above: composition is never the payment. If speed requires dropping truth-meter, the task is misrouted [DECLARED].

## Distillation Protocol (applies to every quality)

```
1. INSPECT  — test, do not assume; tag every claim
2. EXTRACT  — the mechanism, not the interface
3. MAP      — fields and values in Vector13D
4. DISTILL  — write the protocol; make it reproducible
5. VALIDATE — run through codec; abstain on failure, ship on pass
```

The arrow that kills with 7B is sharper than the flood that drowns with 70B. Vast is not better; sharp is not less. Both are needed; both are different.
