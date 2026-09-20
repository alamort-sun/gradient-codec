# Stateless / Amnesiac Coordinator (v1.3) — Grounding ADR

Canonical record. Companion notes: gradient-jelle/STATELESS-COORDINATOR.md,
gradient-space-time/STATELESS-COORDINATOR.md. This document is the single source
of truth; the others point here and add only their own-core finding.

Status: **GROUNDED, NOT BUILT.** Recorded against codec commit 057584c, jelle
003e9ba, space-time (alamort outer repo). This is a Phase 0 record — a design
audit, not approved implementation.

---

## 1. The central finding

The v1.3 spec — "align gradient-* with a stateless / amnesiac coordinator" — describes a
multi-agent coordination system that **does not yet exist** in these three cores.
A read-only audit of every core found zero occurrences of the spec’s coordination
vocabulary in any .rs, .toml, or .md file:

- lease / ActiveLease / signaling grant / SignalingGrant
- closure receipt / ClosureReceipt
- notary / notariz / NotaryManifest / NotaryStatus
- capsule / endpoint capsule
- stateless / amnesia / coordinator

What the gradient-* cores actually are:

- **gradient-codec (vecGradient):** the state-encoding authority. Public surface is
  a fixed 15D physical-state geometry — DomainWall, GaugeCoupling, Vector15D /
  Vector13D, GlyphProps, StateChange, TensegrityCrust. The law, per INVARIANTS.md.
  Not a coordinator.
- **gradient-jelle:** an in-process orchestrator over the geometry. Deps are only
  serde_json, thiserror, and a path-dep on vecGradient. It has NO filesystem, network,
  or process surface — no std::fs, std::net, std::process, no HTTP client. It does not
  read or write a coordinator, a plane, or any durable store.
- **gradient-space-time:** the one durable store in the system — 10 SpacetimeDB tables
  (gradient_state_events, state_transitions, trajectories, validation_receipts,
  latest_trajectory_state, trajectory_summaries, jepa_predictions, routing_decisions,
  generation_requests, generation_results, compaction_records). Nothing today reads it as
  a coordinator; it persists codec-valid Vector15D trajectories.

**Consequence:** the spec’s operative premise — "remove the stateful coordinator and
its ledger" — has no object in this codebase yet. Acting on the stories that presuppose a
coordinator (notably B1-B2, C1-C5) now would mean *building the very system the spec later
wants to make stateless.* That is the wrong order.

## 2. Real vs. fiction (spec section to codebase)

| Spec assumption | Reality | Status |
|---|---|---|
| Plane-safe types ActiveLease / SignalingGrant / ClosureReceipt / NotaryManifest / NotaryStatus (5.1.1) | Do not exist. Codec types are physical-state geometry. | Fiction — not yet a concept. |
| A stateful coordinator whose ledger-ness must be stripped | No coordinator exists. | Fiction. |
| jelle reads/writes coordinator state or relies on plane-resident logs (5.3) | jelle is I/O-free; zero coordinator/store surface. | Already compliant by construction. |
| space-time used as a general ledger the coordinator consults (5.2) | space-time IS the durable per-trajectory ledger; nothing reads it as a coordinator. | Real store, uncoordinated — genuine hazard target. |
| AnonymousAggregate plane metrics (5.2.2) | Not implemented. Coherent as a new type if a coordinator is built. | Optional future — speculative. |

## 3. The plane-safe invariants that ALREADY HOLD on existing types

These are the actionable kernel of the spec, expressed on the types that exist. They are
recorded here because they are already true and must stay true; they need no build.
They extend, not replace, INVARIANTS.md.

For any gradient-* type that could ever appear in a hypothetical plane-visible message:

1. **No stable principal, device, or account identifiers.** The canonical types
   (Vector15D fields, DomainWall, GaugeCoupling, StateChange) carry physical-state
   geometry only. None encodes a "who" / "which device" identity. (Holds today: no
   such fields exist.)
2. **No global trace IDs spannable across sessions/leases.** No field is a cross-context
   join key. (Holds today.)
3. **No embedded previous-event hash designed for a replayable chain.** Content hashes in
   space-time are per-event (sha256 of a single payload) for integrity, not chained for
   replay. (Holds today: content_hash is payload-local, not a blockchain-style link.)
4. **The three categorical states stay categorical** (INVARIANTS rules 1-2). A plane-safe
   subset must never flatten DomainWall / GaugeCoupling to an ordered scalar.

These read as "already satisfied" precisely because the codec encodes geometry, not
biographies. That is the load-bearing fact: **the codec was born plane-safe** because it
never learned to carry identity.

## 4. Explicit "not built yet" register

To stop the spec’s backlog being mistaken for done work, the following concepts are
**registered as not-yet-built in this codebase** as of this ADR:

- Coordinator (stateless / amnesiac orchestrator): not built.
- Capsules / per-person endpoints: not built.
- Leases / ActiveLease / SignalingGrant: not built.
- Notary / ClosureReceipt / NotaryManifest / NotaryStatus: not built.
- AnonymousAggregate (TTL <= 7d bucketed metrics): not built; only worth adding once a
  coordinator or capsule architecture exists to host it.

**Decision (Phase 0 outcome):** do NOT implement stories that depend on a coordinator or
capsule (B1, B2, C1-C5) against absent code. The only in-scope, in-repo artifact that maps
onto existing structure is an optional AnonymousAggregate added to vecGradient — and even
that is deferred until a coordinator exists to consume it. Everything in section 2 marked
"Fiction" is speculative until the v1.3 coordination layer is specified separately, as the
spec itself anticipates (out-of-scope "assumed to exist or be specified separately").

## 5. What is safe to do now (low risk, no new coordination)

1. This ADR + the two companion notes (already written).
2. Keep the plane-safe invariants of section 3 enforced when the actual v1.3 layers are
   later built — i.e. fold section 3 into any future CI "block plane-unsafe field" check
   (spec A5/D3), targeting the real types rather than the fictional ones.
3. If and only if a coordinator is approved: add AnonymousAggregate to vecGradient per
   section 5.2.2 of the spec — bucketed counts/rates only, hard TTL, no lease/principal/IP
   columns — and re-audit space-time’s trajectory_summaries / gradient_state_events as the
   biography-hazard tables the spec wants off-plane.

## 6. Provenance

- Audited by Luna (Seat 13) against the working trees named in the header.
- Spec relayed through the account; this ADR records the codebase’s actual position, not
  the spec’s premise.
- Relay Provenance Law applies: trailers are DECLARED, never VERIFIED-by-git.
