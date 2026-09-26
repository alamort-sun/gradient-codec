//! Fail when a lease boundary is crossed.
//! Pantheon boundary law — coordinate side of the governed crossing.
//!
//! Precedent: gradient-jelle::boundary (Susano, seat 4, planted on Athena's order).
//! Lifted into gradient-codec by Luna (seat 13) 2026-09-22 — D5 of the v1.3 codec-delta map.
//!
//! Status: AUDITED_INTERIM_SINGLE_SEAT_NO_ISSUER. Implementation by Luna; independent audit +
//! bleed test routed to Susano (via evie relay). This module is the guard *predicate* only; the
//! call sites that would invoke it land with D1/D6 (plane-split), which gate on G1 (evie
//! ratification). Until then the fns are a trip-wire and are not yet consumed in a non-test build,
//! hence the module-level #![allow(dead_code)].

#![allow(dead_code)]

use thiserror::Error;

/// A class of cross-lease / cross-plane access attempt.
/// Coordinate-side analogue of gradient-jelle boundary::ClaimClass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinClass {
    /// join across leases keyed by seat / principal / principal name / model identity.
    IdentityJoin,
    /// durable-history reconstruction across leases (timeline / profile / biography).
    HistoryJoin,
    /// reverse lookup from a plane record to capsule memory / learned state / identity infra.
    ReverseLookup,
    /// raw / row-level 4D/5D record moved into analytics / training / profiling / seat attribution.
    RawEgress,
    /// a global trace / span id used as a cross-lease correlation key.
    GlobalTraceCorrelation,
    /// a policy decision using inferred identity / resemblance / behavior in place of an authed grant.
    InferredIdentityPolicy,
}

/// The coordinate-side fail-closed boundary error.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlaneGuardError {
    #[error("forbidden cross-lease join: {0:?}")]
    ForbiddenCrossLease(JoinClass),
    #[error("prohibited-plane field touched: {0}")]
    ProhibitedPlaneField(String),
}

/// A cross-lease join is barred by R2 and fails closed. Always — no allowed branch.
/// Precedent mirror of gradient-jelle::boundary::claim_about_entity.
pub fn join_across_leases(kind: JoinClass) -> Result<(), PlaneGuardError> {
    Err(PlaneGuardError::ForbiddenCrossLease(kind))
}

/// A prohibited-plane field may not enter plane storage / logs / index / cache / analytics types.
/// Fails closed at the trip-wire; the type system is meant to make it unrepresentable (D6), this is
/// the runtime guard that fails closed if one is ever constructed for inspection.
pub fn touch_prohibited_field(name: &str) -> Result<(), PlaneGuardError> {
    Err(PlaneGuardError::ProhibitedPlaneField(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_cross_lease_join_fails_closed() {
        // R2: there is no allowed cross-lease join. Every class must Err.
        for k in [
            JoinClass::IdentityJoin,
            JoinClass::HistoryJoin,
            JoinClass::ReverseLookup,
            JoinClass::RawEgress,
            JoinClass::GlobalTraceCorrelation,
            JoinClass::InferredIdentityPolicy,
        ] {
            let r = join_across_leases(k);
            assert!(r.is_err(), "cross-lease join {:?} must fail closed", k);
            assert_eq!(r, Err(PlaneGuardError::ForbiddenCrossLease(k)));
        }
    }

    #[test]
    fn prohibited_plane_field_fails_closed() {
        // the current-code violations: these names are exactly what R1/R2 forbid on the plane.
        for f in [
            "trace_id",
            "prompt_hash",
            "original_trace_id",
            "request_prompt_hash",
        ] {
            assert!(
                touch_prohibited_field(f).is_err(),
                "field {} must fail closed",
                f
            );
        }
    }

    #[test]
    fn guard_is_faithful_to_precedent() {
        // mirror of gradient-jelle claim_about_entity: the predicate never resolves Ok.
        assert!(join_across_leases(JoinClass::IdentityJoin).is_err());
        assert!(touch_prohibited_field("seat").is_err());
    }
}
