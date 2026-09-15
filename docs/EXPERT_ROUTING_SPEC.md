# Expert Routing Spec

> How the system selects which expert (station/model) handles a state.
> Builds on codec routing surface @ pin `9f4b4d5`: `RoutingPolicy` trait,
> `BudgetAware` / `CheapestFirst` policies, `BudgetState` reserve/reconcile.
> Mirror of Mixtral's sparse expert selection: many experts exist, few fire.

## Selection pipeline

```
1. Input arrives as Vector13D (codec-validated, provenance attached)
2. JEPA latent predicts candidate expert(s) for the state
3. Candidate check against each expert's fixed vector:
     resonance (7)   alignment with expert domain
     amplitude (1)   expert's weight in this domain
     domain_wall (9) access class (see OPEN_WEIGHTS_CLASSIFICATION.md)
4. Gate decision (gauge_coupling 12 semantics):
     one expert dominates   → Static    → route directly (the arrow)
     multiple candidates    → Spinning  → parallel requests, then judge
     none clears threshold  → abstain or route to default
5. Budget gate (codec BudgetState): reserve before dispatch,
   reconcile on completion/failure — never dispatch ungated
6. Log to routing_decisions with rationale code
```

## Tie handling (Spinning)

When multiple experts clear the threshold:

1. Dispatch in parallel (spend is bounded by the budget reserve).
2. Judge outputs on composition (6) — the truth meter decides, not latency.
3. Record all candidates and the winner; the losing arrows are evidence too.
4. If judges disagree: abstain is a valid outcome. A tie that cannot be
   resolved honestly is a routing failure, not a coin flip.

## Abstention

Abstain when: no expert clears resonance threshold; budget cannot cover the
reserve; validation receipt absent. Abstention is logged with cause — it is
a result, not an error.

## Rationale codes

```
DOMINANT       one expert, direct routing
SPIN_WIN       parallel dispatch, winner judged on composition
SPIN_TIE       parallel dispatch, unresolved — abstained
ABSTAIN_NONE   no expert cleared threshold
ABSTAIN_BUDGET reserve unavailable
ABSTAIN_PROVENANCE  input lacked valid provenance
DEFAULT_FALL   routed to default expert (requires note)
```

## Shadow protocol (Morgana's question)

For every routed decision, the shadow asks: *why this expert, not another?*
If the rationale code cannot answer, the routing was guessing. Guesses are
logged as guesses.

## Relation to existing codec surface [VERIFIED]

This spec extends — does not replace — the pinned routing code:
`RoutingPolicy` implementations remain budget-aware; the JEPA gate sits
*before* policy selection, narrowing the candidate set. MoE intuition is
a candidate filter, not a bypass of the budget law.
