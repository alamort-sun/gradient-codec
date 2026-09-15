# Vector13D Arrow Regions

> Regions of the 13-field state space, as defined against codec pin `9f4b4d5`
> (`vector13d/src/lib.rs`). Field indices and aliases verified in source:
> amplitude(1) frequency(2) phase(3) coherence(4) entropy(5) composition(6=truth_meter)
> resonance(7) ozone_buffer(8=lightness) domain_wall(9=connection) su2_polarity(10=hue)
> torsion(11=skew) gauge_coupling(12=rot) closure(13).

All region definitions are **DECLARED** (authored ontology). Field semantics and
observed baseline values are **VERIFIED** @ `9f4b4d5`. Region *usefulness* is
**HYPOTHESIS** until exercised against observed data.

## The Arrow Region

The signature of the huntress: high impact, low scatter, tight grouping.

```
amplitude     (1)  > 0.7
entropy       (5)  < 0.4
coherence     (4)  > 0.6
ozone_buffer  (8)  0.3 – 0.5   (moderate energy, light carry)
torsion       (11) = 0          (upright, direct)
```

Inhabitants: precision filtering, routing decisions, verification, specs.
One line. One target. One arrow.

## The Vast Region

The signature of Athena (station 5): volume retained, scatter allowed.

```
amplitude     (1)  > 0.6
entropy       (5)  > 0.5       (scatter retained on purpose)
ozone_buffer  (8)  > 0.6       (energy-heavy)
frequency     (2)  low          (deep processing per unit)
```

Inhabitants: research sweeps, broad synthesis, scouting.
The Vast Region is not the Arrow Region failing. It is a different weapon.

## Efficiency Region

```
amplitude (1) ÷ entropy (5) = efficiency ratio

efficient:  ratio high  (quality per unit of waste)
bloated:    ratio low   (output volume masking output value)
```

The Arrow Region is the efficiency region at high absolute amplitude.
A state can be efficient and quiet (ratio high, amplitude low) — a good scout,
not a kill shot.

## Precision Region

Subset of the Arrow Region with the truth contract pinned:

```
Arrow Region constraints, plus:
composition (6) > 0.7    (truth preserved)
```

Precision without composition is not precision. It is confidence with nothing
behind the bowstring.

## Fast Region

```
frequency    (2)  high
ozone_buffer (8)  low
amplitude    (1)  moderate (enough, not vast)
```

Fast region states are filters and routers. They are cheap because they carry
nothing extra — speed is the *result* of lightness, not a separate property.

## Slow Region

```
frequency    (2)  low
ozone_buffer (8)  high
amplitude    (1)  high
```

## Open / Closed (discrete, not a region)

Access class is not continuous geometry; it lives in `domain_wall (9)`:

```
Gradient  = open weights  — inspectable, modifiable, local, distillable
Linked    = API access    — callable, opaque
Broken    = closed        — avoided unless no alternative
```

A model's access class constrains which protocols may touch it:
JEPA latent extraction, hidden-state hooks, and distillation require Gradient.
See `OPEN_WEIGHTS_CLASSIFICATION.md`.

## Observed anchors (VERIFIED)

The only observed states on record (n1 → n2, @ `9f4b4d5`):

```
n1: su2_polarity 331.4° (pink), entropy 0.08, torsion −50°, Spinning
n2: su2_polarity 320°  (purple), entropy 0.03, torsion −5°,  Oscillating
domain_wall constant Linked across the transition
```

Note: observed n1/n2 entropy values (0.08 → 0.03) already sit deep inside the
Arrow Region's entropy bound. The baselines are arrow-shaped data. Regions
remain DECLARED until more observed states exercise their boundaries —
region edges are hypotheses with coordinates.

## Validation rule

Any state claimed to be "in" a region must pass codec validation at the pinned
commit and carry provenance. Region membership is computed, never asserted.
