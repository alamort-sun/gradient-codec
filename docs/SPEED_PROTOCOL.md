# Speed Protocol

> When fast, when slow, and how the difference is measured.
> Fields @ codec pin `9f4b4d5`: frequency (2), ozone_buffer (8),
> amplitude (1), composition (6).

## The law

The arrow is fast because it is light. It is light because it carries nothing
extra. Speed is not a feature; it is the result of carrying only what is
needed. **Composition (6) is never the payment for speed.**

## Task classification

```
FAST  (route to Arrow / fast models):
  filter, route, verify, scout

SLOW  (route to Vast / deep models):
  analyze, generate, synthesize, reconcile multi-document
```

Classification is task-first, station-second: a sharp task routed to a vast
station wastes energy; a vast task routed to a sharp station misses depth.
Both are routing failures and are logged as such.

## Measurement

```
frequency (2)    = states cycled per unit time
ozone_buffer (8) = energy consumed per cycle (lightness; default 0.5)

speed_efficiency = frequency ÷ ozone_buffer
```

Measured per model and per pipeline segment. Logged in `routing_decisions`
alongside the access class.

## Dispatch rules

1. Time-sensitive task → prioritize speed, keep composition floor.
2. Quality-sensitive task → prioritize depth, budget permitting.
3. Never drop validation to meet a latency target. A fast wrong answer is
   the most expensive output there is.
4. Budget gate always applies (codec `BudgetState` reserve/reconcile) —
   speed does not exempt a dispatch from the reserve.

## When slow is better

Deep analysis, creative generation, complex reasoning, multi-document
synthesis — low frequency (2), high ozone_buffer (8), high amplitude (1).
The Vast Region exists because some targets cannot be taken with one arrow.

## Shadow

What speed is actually just skipping? Any speedup observed together with a
composition (6) drop. The audit is one comparison: if composition fell, the
"speed" was theft, and the routing is re-classified as a miss.
