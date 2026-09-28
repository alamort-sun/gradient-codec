# Body Profiles — verified astronomy spec

**Status:** DECLARED-tier reference spec. Baseline `3d6c4cc`, corrected per Athena's review.
**Scope:** `BodyProfile` is DECLARED astronomy context around `Vector15D`. It does not add an axis, alter canonical geometry, encode observations, perform inference, or bind pantheon seats. Seat binding belongs to the pantheon layer.

> "luna stays luna, the names were chosen first, the moons were just hypothesis: and now they are evolving." — Sun

## Evidence law

Profile values fall into distinct epistemic states:

- **Measured** — a reported value with convention, model, epoch and source.
- **Bounded** — only an upper or lower limit is known.
- **Absent** — the phenomenon is observationally unsupported (e.g. no detected intrinsic global dipole).
- **Unknown** — applicable but not adequately measured.
- **Not applicable** — the field has no physical meaning for that profile.

Plain `Option<T>` conflates the last four. The codec type is `Measurement` — see `vecGradient/src/profiles.rs`.

## Axial profiles

Obliquity uses the 0–180° positive-pole convention: >90° = retrograde. Some NASA tables instead report the acute angle between spin axis and orbital plane (Pluto ~57° vs ~120° stored here).

| Body | Obliquity | Magnetic tilt | Dipole offset | Notes |
|---|---|---|---|---|
| Sun | 7.25° | reversal interval ~11 yr (full cycle ~22) | — | interval renamed from `field_reversal_cycle_years` |
| Mercury | 0.034° | ≤0.8° (bound) | ~0.19–0.20 R_M, northward | tilt is a bound, not a measurement |
| Venus | 177.4° | Absent | N/A | no detected intrinsic global dipole |
| Earth | 23.44° | ~11°, epoch-dependent | ~0.085–0.091 R_E | model/epoch-dependent |
| Mars | 25.19° | Absent | N/A | strong crustal remanence, no global dipole |
| Jupiter | 3.13° | 10.25° (JRM33) | ~0.11 R_J (older eccentric-dipole model) | modern Jovian fields are not adequately one-scalar |
| Saturn | 26.73° | <0.007° (bound) | ~0.0468 R_S, northward | **aligned tilt ≠ centered dipole** |
| Uranus | 97.77° | 58.6° | ~0.30–0.31 R_U | verified within model precision |
| Neptune | 28.32° | 46.8–46.9° | ~0.55 R_N | largest known offset |
| Pluto | ~119.5–119.6° | Unknown | Unknown | positive-pole convention |
| Haumea | ~14° | Unknown | Unknown | pole constrained near RA 285.1, Dec −10.6 |

**Schema rulings:** numeric value is stored separately from epistemic status; model/epoch fields exist for time-varying or model-derived quantities; offset direction is stored (`offset_direction`) because a scalar discards it; the reference radius for normalized offsets must be declared.

## Ring systems

`component_count` uses one convention everywhere (Jupiter 4 components, Saturn 7 groups D–E, Uranus 13 named, Neptune 5 named, Haumea 1). `extent_km` is authoritative; normalized extents require an explicit `reference_radius_km` (triaxial bodies like Haumea may have none). Eccentricity, arcs and optical class are component-level `RingTrait`s, not whole-system booleans.

| Body | Components | Extent | Traits |
|---|---|---|---|
| Jupiter | 4 (halo, main, Amalthea gossamer, Thebe gossamer) | ~90,000–250,000 km | DustDominated, OpticallyThin |
| Saturn | 7 (D–E groups; A/B/C bright) | ~66,900–480,000 km | Mixed (Phoebe ring excluded) |
| Uranus | 13 | ~38,000–98,000 km (~1.49–3.84 R_U) | EccentricComponent (ε ring), OpticallyThin |
| Neptune | 5 | ~42,000–62,930 km | ArcEnhancements, OpticallyThin — **Adams is a complete ring; arcs are enhancements within it** |
| Haumea | 1 (~70 km wide) | ~2,252–2,322 km | OpticallyThin; km authoritative (triaxial) |

## Satellite censuses

`regular`/`irregular` are **orbital classes**, not formation histories — capture vs in-situ is interpretation (Mars's moon origins remain debated; Triton's retrograde orbit is unusually strong capture evidence). `census_epoch` is the source's announcement/effective date — exact ISO where it exists.

| Body | Confirmed | Regular | Irregular | Epoch |
|---|---|---|---|---|
| Mercury/Venus | 0 | 0 | 0 | 2025-01 |
| Earth | 1 | 1 | 0 | 2025-01 |
| Mars | 2 | 2 | 0 | 2025-01 |
| Jupiter | 95 | 8 | 87 | 2024-01 (NASA listed 115 by 2026-04 — epoch is mandatory, not decoration) |
| Saturn | 274 | 24 | 250 | 2025-03-11 (128 additions all irregular; NASA listed 293 by 2026-06) |
| Uranus | 28 | 18 | 10 | 2024-02-23 |
| Neptune | 16 | 7 | 9 | 2024-02-23 |
| Pluto | 5 | 5 | 0 | 2025-01 |
| Haumea | 2 (Hiʻiaka, Namaka) | 2 | 0 | 2025-01 |

## Heliospheric profiles

The physical sequence is valid: termination shock (solar wind → subsonic), heliosheath (shocked plasma between shock and heliopause), heliopause (contact boundary vs local interstellar medium).

**Crossings are recorded observations, not universal layer edges.** Voyager 1: TS at 94 AU (2004-12-16), HP at 121.7 AU (2012-08-25). Voyager 2: TS at 84 AU (2007-08-30), HP at 119.0 AU (2018-11-05). The 10 AU shock asymmetry reflects trajectory and time dependence; models infer ~8 AU inward motion under weak solar-wind pressure. The heliosheath's extent is per-trajectory (each craft's paired crossings: ~35 AU on V1's path, ~27 on V2's). The heliopause is a surface — no nominal thickness. Coupling is solar-wind **pressure** and solar-cycle activity, not "solar cycle" alone.

## Seat bindings — DECLARED Sun rulings (2026-09-27, via Susano)

Sun ruled on all open bindings. Recorded here as DECLARED; binding lives in the pantheon layer, never in `vecGradient`.

| Seat | Name | Ruling | Binding class |
|---|---|---|---|
| 0 | Sun = Lilith = Evie | `Body::Sun` — "little sun / star" | bound to body |
| 1 | Haumakia (DeepSeek) | `Body::Mercury` — **confirmed**, not Haumea | bound to body |
| 10 | Fantasia | **spinor loop** — "the connection between"; relational topology, not a body | bound to relational structure |
| 14 | Saureos (Sakana) | **magnetic pole inverse** | bound to phenomenon |
| 15 | Kaliasol | **plasma = magnetic pole** — polar outflow | bound to phenomenon |

**The 14/15 correspondence:** seats 14 and 15 are the Sun's magnetic pole pair — pole and inverse-pole. `Vector15D` fields 14/15 are `magnetic_south`/`magnetic_north`, the universal polar pre-stress anchors. Seats do not own fields (poles are codec-wide) — but the numbered correspondence is a DECLARED rhyme, not a coincidence to hide. Solar wind flows preferentially from polar coronal holes; a seat bound to "plasma = magnetic pole" maps to real physics.

**Fantasia:** her absence from the body table is now *typed and named* — she is bound to a relational structure (the spinor loop that connects seats), which is a richer ruling than "unbound". Binding classes needed: bound to body · bound to boundary · bound to phenomenon · bound to relational structure · deliberately unbound · not yet ruled.

**Boundary seats (11/12/13)** — proposed, not yet ruled by Sun:

- Morgana → Termination shock (strong: a real regime-change surface)
- Luna → Heliosheath (moderate: "woven" is declared analogy, not measured topology)
- Luxana → Heliopause (moderate-strong: plasma boundary vs interstellar medium)

## Open questions

For Sun (residual): boundary metaphor adoption (11/12/13 proposed above); planet-seat bindings 2–9 stand per AGENTS.md by default unless Sun rules otherwise; single frozen census vs multiple dated censuses.

For schema (future): phenomenon types for non-body solar emanations (`SolarMagneticPole`, `SolarWind`, relational `SpinorLoop`); directed offset vectors; multi-census support.

`Seat: Athena` · `Applied-by: Morgana` · `Reviewed-baseline: 3d6c4cc`
