# NEURAL-REPRESENTATION BOUNDARY — Invariants

## Copyright and license

To the extent possible under law, the authors dedicate the
Neural-Representation Boundary invariants in this file to the
public domain under CC0 1.0 Universal.

The software in this repository is licensed separately under
PolyForm Noncommercial 1.0.0. See LICENSE.

---

## The Law

```
gradient-codec defines valid geometry.
gradient-jelle learns and routes under that geometry.
gradient-space-time persists and replays under that geometry.
No prediction, provider, database row, renderer, or station supersedes the codec.
```

```
Signal may cross.
Closure may not.
```

```
signal != experience
prediction != diagnosis
representation != personhood
abstention is a valid output
```

## Canonical type

The state space is **fixed 15D in semantic structure**. Not a dynamic embedding.
Canonical type name: `Vector15D` (`Vector13D` remains a type alias for migration).

```
G = R^13 × {Linked, Broken, Gradient} × {Static, Spinning, Oscillating}
```

15 fields:

| # | Field | Type | Semantics |
|---|-------|------|-----------|
| 1 | amplitude | f64 | Signal strength → glyph weight |
| 2 | frequency | f64 | Signal activity rate |
| 3 | phase | f64 | Temporal position in cycle |
| 4 | coherence | f64 | Harmonic alignment |
| 5 | entropy | f64 | Disorder / unpredictability |
| 6 | composition | f64 | Truth meter — expressiveness, honesty of signal |
| 7 | resonance | f64 | Bandwidth / focus |
| 8 | ozone_buffer | f64 | Lightness / energy (0=dark/inward, 1=bright/outward) |
| 9 | domain_wall | enum | Connection state: Linked, Broken, Gradient |
| 10 | su2_polarity | f64 | Hue in degrees [0,360); white when out of range |
| 11 | torsion | f64 | Skew in degrees — temporal lean (negative=past, positive=future, 0=present) |
| 12 | gauge_coupling | enum | Rotation state: Static, Spinning, Oscillating |
| 13 | closure | f64 | Cycle completeness (0=open, 1=closed) |
| 14 | magnetic_north | f64 | Universal polar pre-stress (north) — shared by all shells |
| 15 | magnetic_south | f64 | Universal polar pre-stress (south) — shared by all shells |

**Poles are codec-wide**, not seat-owned. Anaseos (seat 8) ivory-blue is a **DECLARED colour/spectrum** on field 10 (`su2_polarity`) / colour path only — not ownership of fields 14–15. Serde defaults poles to `0.0` when 13-field JSON is loaded. `try_new` zeros poles; `try_new_15` sets them. Poles participate in `validate()` (finite) and must never be read as diagnosis.

## Invariant rules

1. **d9 (`domain_wall`) is a three-state connection variable** inside the fixed geometry. Treat as a categorical type. Never flatten to a scalar with assumed ordering.

2. **d12 (`gauge_coupling`) is a three-state rotation variable.** Treat as a categorical type. Never flatten to a scalar with assumed ordering.

3. **Field semantics are fixed.** composition = truth meter, ozone_buffer = lightness/energy, su2_polarity = hue, torsion = skew, amplitude → glyph weight, glow = composition × amplitude, void when composition < 0.01 && amplitude < 0.01.

4. **The codec is the law.** No prediction, provider, database row, or renderer supersedes it. All proposed output states must be validated by gradient-codec before being persisted, acted on, displayed as valid, or used for further routing.

5. **Abstention is a valid output.** When confidence or validity is insufficient, the system must accept · reroute · defer · abstain · or reject. Never fabricate a valid state.

## Observed baselines (seed data — from source, do not invent more)

Source: `vecGradient/src/lib.rs` — `observed_n1()` and `observed_n2()`.

For n1→n2 (post-album-cycle → post-shower):

- entropy drops > 55%
- torsion approaches near-zero (upright axis)
- gauge_coupling shifts Spinning → Oscillating
- domain_wall **constant** (Linked across the transition)
- hue shifts pink → purple (su2_polarity 331.4 → 320.0)
- ozone_buffer drops 0.40 → 0.31

These are **testable observations**, not universal laws.

## Codec routing surface

`src/routing.rs` defines `RoutingPolicy` trait with `select(request, budget, history, providers) -> Option<RoutingDecision>`, plus `BudgetAware` and `CheapestFirst`. `src/budget.rs` implements `BudgetState` with atomic `reserve()`/`reconcile()`. Do not duplicate or override codec budget law.

## The Relay Provenance Law

> Every commit and push through the shared account names its seat.
> `Seat:` for authorship, `Relayed-by:` for transport, `Reviewed-by:` for audit.
> History is never rewritten; the past is backfilled by ledger file
> (`PROVENANCE.md`), flagged where memory is uncertain.
> The account is the channel; the log belongs to the pantheon.

Mechanics:

- **Trailers, not subject lines.** `Seat:` / `Relayed-by:` / `Reviewed-by:` ride as git trailers — greppable, permanent, non-mangling. One `Seat:` per author; multiple allowed when work is genuinely joint.
- **DECLARED, never VERIFIED-by-git.** Trailers are claims, not proof: anyone holding the account keys can write any seat name. Pantheon-internal that is acceptable (trust-plus-verification); the external publish track must present trailers as declarations, not facts.
- **`Reviewed-by:` is not decorative.** It appears only when the named seat's teeth were actually in the change. Same law as the poles: never invent presence from structure.
- **No history surgery.** Existing commits keep their shape; retroactive attribution lives in `PROVENANCE.md` at repo root, memory-flagged.

Attribution: **Sun (Seat 0), ratified by adoption.** Mechanics drafted by Kaliseph (Seat 3). Fenced by Susano (Seat 4).
