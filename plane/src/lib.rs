//! Saraswati A2 plane-safe identifier / digest newtypes.
//!
//! Design law: a type is plane-safe when the banned join is **inexpressible**.
//! These are **named-field** newtypes (never `String` aliases, never tuple
//! structs — SpacetimeDB 1.12 proc-macro panics on tuple-struct field.ident).
//!
//! Authority: `agent/specs/p1-codec-schema-lane-saraswati.md` §2;
//! `agent/specs/stateless-coordinator-v1.3` SCHEMA TODO (space-time).
//!
//! Capsule runtime is **out of scope** here — only the opaque ID/digest belt.

use serde::{Deserialize, Serialize};

#[cfg(feature = "spacetimedb")]
use spacetimedb::SpacetimeType;

/// Opaque per-purpose lease id. Never reused. Not a trajectory / seat / principal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(SpacetimeType))]
#[cfg_attr(feature = "spacetimedb", sats(name = "LeaseId"))]
pub struct LeaseId {
    pub value: u64,
}

impl LeaseId {
    pub const fn new(value: u64) -> Self {
        Self { value }
    }
}

/// sha256 over a normalized act commitment (hex, lowercase preferred).
/// Binds an act without describing provider / prompt / cost.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(SpacetimeType))]
#[cfg_attr(feature = "spacetimedb", sats(name = "ActDigest"))]
pub struct ActDigest {
    /// 64-char hexadecimal sha256.
    pub hex: String,
}

/// sha256 of capsule content — binds without storing capsule bytes.
/// Distinct from [`ActDigest`] so a capsule cannot be passed as an act.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(SpacetimeType))]
#[cfg_attr(feature = "spacetimedb", sats(name = "CapsuleDigest"))]
pub struct CapsuleDigest {
    pub hex: String,
}

/// Monotonic policy version fence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(SpacetimeType))]
#[cfg_attr(feature = "spacetimedb", sats(name = "PolicyVersion"))]
pub struct PolicyVersion {
    pub value: u32,
}

impl PolicyVersion {
    pub const fn new(value: u32) -> Self {
        Self { value }
    }
}

/// Fenced security epoch (monotonic root).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(SpacetimeType))]
#[cfg_attr(feature = "spacetimedb", sats(name = "SecurityEpoch"))]
pub struct SecurityEpoch {
    pub value: u64,
}

impl SecurityEpoch {
    pub const fn new(value: u64) -> Self {
        Self { value }
    }
}

/// Opaque receipt id for a [`ClosureCode`] disposition (never a trajectory key).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(SpacetimeType))]
#[cfg_attr(feature = "spacetimedb", sats(name = "ReceiptId"))]
pub struct ReceiptId {
    pub value: u64,
}

impl ReceiptId {
    pub const fn new(value: u64) -> Self {
        Self { value }
    }
}

/// Fresh manifest id per window (never reused across windows).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(SpacetimeType))]
#[cfg_attr(feature = "spacetimedb", sats(name = "ManifestId"))]
pub struct ManifestId {
    pub value: u64,
}

impl ManifestId {
    pub const fn new(value: u64) -> Self {
        Self { value }
    }
}

/// Closed analytics window: start (unix micros) + length (seconds).
/// Micros keep this crate free of chrono / SpacetimeDB `Timestamp` at the type layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(SpacetimeType))]
#[cfg_attr(feature = "spacetimedb", sats(name = "Window"))]
pub struct Window {
    pub start_micros: i64,
    pub len_secs: u64,
}

/// Suppression notes (rare-category / timing / joinability) — mandatory on manifests.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(SpacetimeType))]
#[cfg_attr(feature = "spacetimedb", sats(name = "SuppressionSpec"))]
pub struct SuppressionSpec {
    pub value: String,
}

impl SuppressionSpec {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

/// Closure disposition for an act (Saraswati A2).
/// Former space-time `ValidationOutcome` / `GenerationStatus` vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(SpacetimeType))]
#[cfg_attr(feature = "spacetimedb", sats(name = "ClosureCode"))]
pub enum ClosureCode {
    #[default]
    Accepted,
    Rerouted,
    Abstained,
    Rejected,
}

/// Hex-encoded sha256 digest length.
pub const DIGEST_HEX_LEN: usize = 64;

/// Reject digests that are not hex sha256.
pub fn require_digest_hex(label: &str, digest: &str) -> Result<(), String> {
    if digest.len() != DIGEST_HEX_LEN {
        return Err(format!(
            "{label}: expected {DIGEST_HEX_LEN}-char hex digest, got len {}",
            digest.len()
        ));
    }
    if !digest.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("{label}: digest must be hexadecimal"));
    }
    Ok(())
}

impl ActDigest {
    pub fn from_hex(hex: impl Into<String>) -> Result<Self, String> {
        let hex = hex.into();
        require_digest_hex("act_digest", &hex)?;
        Ok(Self { hex })
    }

    pub fn as_hex(&self) -> &str {
        &self.hex
    }
}

impl CapsuleDigest {
    pub fn from_hex(hex: impl Into<String>) -> Result<Self, String> {
        let hex = hex.into();
        require_digest_hex("capsule_digest", &hex)?;
        Ok(Self { hex })
    }

    pub fn as_hex(&self) -> &str {
        &self.hex
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lease_id_is_not_receipt_id() {
        // Distinct newtypes — cannot coerce without explicit field access.
        let lease = LeaseId::new(1);
        let receipt = ReceiptId::new(1);
        assert_eq!(lease.value, receipt.value);
        let _ = (lease, receipt);
    }

    #[test]
    fn act_digest_accepts_sha256_hex() {
        let d = ActDigest::from_hex("a".repeat(64)).expect("hex ok");
        assert_eq!(d.as_hex().len(), 64);
    }

    #[test]
    fn act_digest_rejects_short() {
        let err = ActDigest::from_hex("deadbeef").expect_err("short");
        assert!(err.contains("expected 64"));
    }

    #[test]
    fn capsule_digest_rejects_non_hex() {
        let err = CapsuleDigest::from_hex("g".repeat(64)).expect_err("non-hex");
        assert!(err.contains("hexadecimal"));
    }

    #[test]
    fn act_and_capsule_digests_are_distinct_types() {
        let a = ActDigest::from_hex("b".repeat(64)).unwrap();
        let c = CapsuleDigest::from_hex("b".repeat(64)).unwrap();
        assert_eq!(a.as_hex(), c.as_hex());
        // No From impl between them — purpose-scoped equality only via explicit hex.
        let _ = (a, c);
    }

    #[test]
    fn window_and_suppression_construct() {
        let w = Window {
            start_micros: 0,
            len_secs: 86_400,
        };
        let s = SuppressionSpec::new("floor=100");
        assert_eq!(w.len_secs, 86_400);
        assert_eq!(s.value, "floor=100");
    }

    #[test]
    fn closure_code_default_accepted() {
        assert_eq!(ClosureCode::default(), ClosureCode::Accepted);
    }
}
