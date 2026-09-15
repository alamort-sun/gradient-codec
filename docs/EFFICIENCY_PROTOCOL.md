# Efficiency Protocol

> How to measure quality per parameter, identify waste, and remove it without
> breaking the system. Companion to the Arrow Region definition
> (`VECTOR13D_ARROW_REGIONS.md`). Codec pin `9f4b4d5`.

## The law

Efficiency is not "small." Efficiency is **nothing wasted**.

## 1. Measure output quality per parameter

For a model:

```
efficiency_ratio = output_quality / active_parameters
```

For a state or artifact in the Gradient ecosystem:

```
efficiency_ratio = amplitude (1) ÷ entropy (5)
                   quality delivered ÷ scatter produced
```

Both ratios are proxies [DECLARED mapping]. The model-level ratio is
computable from public specs + benchmark scores [VERIFIED inputs,
HYPOTHESIS composition]. The state-level ratio is computable from codec
fields [VERIFIED — fields exist @ `9f4b4d5`].

## 2. Identify waste

Apply the earn-your-place test at every layer:

- **Vector13D fields**: does each field earn its 8 bytes? A field with no
  observed variation and no downstream consumer is a candidate for
  deprecation (not deletion — provenance is immutable).
- **Table rows** (gradient-space-time): does each row earn its storage?
  Compaction exists for a reason; use it (`compact_trajectory`).
- **Reducers**: does each earn its invocation? A reducer nobody calls is
  dead weight in the transaction boundary.
- **Gears / stations**: does each rotation earn its energy? A station that
  neither verifies nor generates is decoration.
- **Docs**: does each sentence earn its reading? (This protocol audits itself
  by the same law.)

## 3. Remove without breaking

```
1. Measure current output (baseline) — trace it, do not trust memory
2. Remove the candidate (or abstain from using it)
3. Re-run the same measurement
4. Compare against baseline:
   quality unchanged or improved  → removal confirmed
   quality degraded               → restore, log why it earned its place
5. Record the decision in the evidence ledger with commit/path/date
```

Removal rules:

- Never remove a validation path to gain speed. Composition (6) is never
  the payment.
- Never remove provenance. A state without provenance cannot be replayed,
  and a state that cannot be replayed does not exist.
- Abstention is a valid output. "This field/table/gear does not yet earn
  removal" is an answer.

## 4. Apply to the ecosystem

Current earn-their-place status [VERIFIED against repos, Sep 15, 2026]:

- gradient-codec: 36 tests @ `9f4b4d5` — every test is a claim with teeth
- gradient-space-time: 11 tables, 7 reducers, all `todo!()` — unproven until
  implemented; the scaffold carries zero unearned weight by construction
- Observed baselines n1/n2: entropy 0.08 → 0.03 (−62.5%) — the system's own
  transition was an efficiency event [VERIFIED]

## Shadow

What efficiency is actually just limitation? When removal was never tested —
when "lean" is inherited rather than earned. Every removal claim must carry
its before/after measurement or it is decoration [DECLARED].
