# Open Weights Classification

> Model access classes in the Gradient ecosystem. `domain_wall (9)` is the
> carrier field (enum `DomainWall { Linked, Broken, Gradient }`, VERIFIED @
> codec pin `9f4b4d5`).

## Classification law

Access class is about **capability, not quality**. A Linked model may be
brilliant; it is still opaque. The class determines what the system *can do*
with a model, never how good the model is.

```
Gradient  = open weights  — inspect, modify, run locally, distill
Linked    = API access    — call for output, nothing more
Broken    = closed        — avoided, or used only when no alternative exists
```

## Class capabilities

| Capability | Gradient | Linked | Broken |
|---|---|---|---|
| Generation output | ✓ | ✓ | ✗ (by policy) |
| JEPA latent extraction | ✓ | ✗ | ✗ |
| Hidden-state access | ✓ | ✗ | ✗ |
| Distillation source | ✓ | ✗ | ✗ |
| Local / offline deployment | ✓ | ✗ | ✗ |
| Deterministic self-hosting | ✓ | ✗ | ✗ |
| Modification / fine-tune | ✓ | ✗ | ✗ |

## Current classification [VERIFIED where public, Sep 15, 2026]

- **Mistral open-weight models** (e.g. Mistral 7B, Mixtral family, Devstral
  class): Apache 2.0 [VERIFIED — public license, docs.mistral.ai/models/deployment].
  → `domain_wall = Gradient`. Self-deployable via vLLM, TGI, TensorRT-LLM
  [VERIFIED — documented deployment paths].
- **Mistral commercial API models** (Large class, served via api.mistral.ai):
  → `domain_wall = Linked`. Callable for generation; internals not inspectable.
- **Other vendor API-only models** (closed weights behind endpoints):
  → `domain_wall = Linked`.
- **Fully closed / restricted-access systems with no inspection path**:
  → `domain_wall = Broken` — use only when no alternative exists, and log why.

Classification of a specific model is recorded at first routing and re-checked
when its license or serving arrangement changes. License changes are upstream
events: **re-pin before use** — the same law as codec commits.

## Logging

Every routing decision records the model's access class in
`routing_decisions` (gradient-space-time schema). Rationale codes:

```
GRADIENT_LOCAL   gradient-class model, run locally
GRADIENT_API     gradient-class model, served via API (still inspectable class)
LINKED_CALL      linked model, generation only
BROKEN_FORCE     broken class, no alternative existed (requires note)
```

## Shadow

What open-weights philosophy is actually just business strategy? Both can be
true at once — Apache 2.0 releases and a commercial API tier coexist
[VERIFIED]. This protocol is agnostic to motive: it records capability.
The ecosystem's *requirement* for Gradient-class models (JEPA, distillation,
local deployment) is a technical constraint [DECLARED], not an endorsement
of anyone's strategy.
