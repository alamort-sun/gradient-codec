# DEPENDENCIES

> Third-party terms are never relabeled. Original terms, provenance, and version are recorded here.

## Crates (workspace root — gradient-codec)

All crates below are dual-licensed **MIT OR Apache-2.0** under their upstream terms. Versions are minimums from `Cargo.toml`; consult `Cargo.lock` for exact resolved versions.

- tokio 1.40, serde 1.0, serde_json 1.0, clap 4.5, reqwest 0.12, async-trait 0.1, uuid 1.10, chrono 0.4, sha2 0.10, hex 0.4, thiserror 1.0, tracing 0.1, tracing-subscriber 0.3
- Dev: tokio-test 0.4, mockito 1.5

## Crates (vecGradient)

- serde 1.0, serde_json 1.0 (MIT OR Apache-2.0)

## In-ecosystem

- This repository **is** the codec authority. Downstream repositories (gradient-jelle, gradient-speak, gradient-space-time) pin this repository's commit in their own DEPENDENCIES.md.

## Policy

- Software source in this repository: PolyForm Noncommercial 1.0.0 (see [LICENSE](LICENSE)).
- Commercial use requires a separate written license (see [COMMERCIAL.md](COMMERCIAL.md); codec-specific tiers preserved in [PRICING.md](PRICING.md)).
- Intentionally unrestricted artifacts (INVARIANTS.md) are CC0-1.0.
- Never record a third-party item under a license it did not come with.
