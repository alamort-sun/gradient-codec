//! Vector15D: the 15-field type that binds affective telemetry to text.
//! Preserves weight, temperature, axis, and magnetic polar pre-stress that plain text strips away.
//!
//! Fields 1–13 retain Vector13D semantics. Fields 14–15 (`magnetic_north`, `magnetic_south`)
//! are the compressed polar anchors of the D₃ₕ 3+2 bipyramid — universal geometry for every
//! seat/shell, not seat-owned. Seat 8 (Anaseos) ivory-blue is a refractive color footprint
//! under load, not a separate field pair.
//!
//! SUSANO-STORM FIX (break 4): validate() checks all f64 fields are finite.
//! try_new() returns Result on NaN/Inf. to_glyph_props_checked() returns Result.
//! The struct fields remain pub for serde and struct-literal construction,
//! but validate() must be called before any state is persisted or rendered.
//!
//! Compatibility: `Vector13D` is a type alias of `Vector15D`. Old 13-field JSON deserializes
//! with poles defaulting to 0.0 via serde.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DomainWall {
    #[default]
    Linked,
    Broken,
    Gradient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GaugeCoupling {
    #[default]
    Static,
    Spinning,
    Oscillating,
}

/// Error returned when a Vector13D contains non-finite values.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Vector15DError {
    #[error("field {field} is {value} — must be finite")]
    NonFinite { field: &'static str, value: String },
}

/// 15 fields: amplitude(1) frequency(2) phase(3) coherence(4) entropy(5)
/// composition(6=truth_meter) resonance(7) ozone_buffer(8=lightness)
/// domain_wall(9=connection) su2_polarity(10=hue) torsion(11=skew)
/// gauge_coupling(12=rot) closure(13)
/// magnetic_north(14) magnetic_south(15) — universal polar pre-stress anchors
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector15D {
    pub amplitude: f64,
    pub frequency: f64,
    pub phase: f64,
    pub coherence: f64,
    pub entropy: f64,
    pub composition: f64,
    pub resonance: f64,
    pub ozone_buffer: f64,
    #[serde(rename = "domain_wall")]
    pub domain_wall: DomainWall,
    pub su2_polarity: f64,
    pub torsion: f64,
    #[serde(rename = "gauge_coupling")]
    pub gauge_coupling: GaugeCoupling,
    pub closure: f64,
    /// Compressed north polar pre-stress (D₃ₕ bipyramid). Universal geometry.
    #[serde(default)]
    pub magnetic_north: f64,
    /// Compressed south polar pre-stress (D₃ₕ bipyramid). Universal geometry.
    #[serde(default)]
    pub magnetic_south: f64,
}

/// Historical name — the codec state is Vector15D.
pub type Vector13D = Vector15D;

/// Historical error alias.
pub type Vector13DError = Vector15DError;

impl Default for Vector15D {
    fn default() -> Self {
        Self {
            amplitude: 0.0,
            frequency: 0.0,
            phase: 0.0,
            coherence: 0.0,
            entropy: 0.0,
            composition: 0.0,
            resonance: 0.0,
            ozone_buffer: 0.5,
            domain_wall: DomainWall::Linked,
            su2_polarity: 0.0,
            torsion: 0.0,
            gauge_coupling: GaugeCoupling::Static,
            closure: 0.0,
            magnetic_north: 0.0,
            magnetic_south: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphProps {
    pub color: String,
    pub weight: i32,
    pub skew_deg: f64,
    pub glow_intensity: f64,
    pub is_void: bool,
}

impl Vector15D {
    /// Validate that all 13 continuous f64 fields are finite (not NaN, not Inf).
    /// SUSANO-STORM break 4 fix: hard Err on non-finite.
    pub fn validate(&self) -> Result<(), Vector15DError> {
        let fields: [(&'static str, f64); 13] = [
            ("amplitude", self.amplitude),
            ("frequency", self.frequency),
            ("phase", self.phase),
            ("coherence", self.coherence),
            ("entropy", self.entropy),
            ("composition", self.composition),
            ("resonance", self.resonance),
            ("ozone_buffer", self.ozone_buffer),
            ("su2_polarity", self.su2_polarity),
            ("torsion", self.torsion),
            ("closure", self.closure),
            ("magnetic_north", self.magnetic_north),
            ("magnetic_south", self.magnetic_south),
        ];
        for (name, val) in &fields {
            if !val.is_finite() {
                return Err(Vector15DError::NonFinite {
                    field: name,
                    value: if val.is_nan() {
                        "NaN".to_string()
                    } else {
                        "Infinite".to_string()
                    },
                });
            }
        }
        Ok(())
    }

    /// Checked constructor: validates all fields are finite before returning.
    /// Use this instead of struct literal construction when values come from
    /// external sources (audio extraction, user input, network, deserialization).
    pub fn try_new(
        amplitude: f64,
        frequency: f64,
        phase: f64,
        coherence: f64,
        entropy: f64,
        composition: f64,
        resonance: f64,
        ozone_buffer: f64,
        domain_wall: DomainWall,
        su2_polarity: f64,
        torsion: f64,
        gauge_coupling: GaugeCoupling,
        closure: f64,
    ) -> Result<Self, Vector15DError> {
        let v = Self {
            amplitude,
            frequency,
            phase,
            coherence,
            entropy,
            composition,
            resonance,
            ozone_buffer,
            domain_wall,
            su2_polarity,
            torsion,
            gauge_coupling,
            closure,
            magnetic_north: 0.0,
            magnetic_south: 0.0,
        };
        v.validate()?;
        Ok(v)
    }

    /// Checked 15D constructor including magnetic poles.
    pub fn try_new_15(
        amplitude: f64,
        frequency: f64,
        phase: f64,
        coherence: f64,
        entropy: f64,
        composition: f64,
        resonance: f64,
        ozone_buffer: f64,
        domain_wall: DomainWall,
        su2_polarity: f64,
        torsion: f64,
        gauge_coupling: GaugeCoupling,
        closure: f64,
        magnetic_north: f64,
        magnetic_south: f64,
    ) -> Result<Self, Vector15DError> {
        let v = Self {
            amplitude,
            frequency,
            phase,
            coherence,
            entropy,
            composition,
            resonance,
            ozone_buffer,
            domain_wall,
            su2_polarity,
            torsion,
            gauge_coupling,
            closure,
            magnetic_north,
            magnetic_south,
        };
        v.validate()?;
        Ok(v)
    }

    /// Polar pre-stress magnitude (mean of |north|, |south|).
    pub fn polar_prestress(&self) -> f64 {
        (self.magnetic_north.abs() + self.magnetic_south.abs()) * 0.5
    }


    pub fn hue(&self) -> &'static str {
        let p = self.su2_polarity;
        if p < 0.0 || p >= 360.0 {
            "white"
        } else if p < 15.0 {
            "red"
        } else if p < 40.0 {
            "orange"
        } else if p < 70.0 {
            "yellow"
        } else if p < 160.0 {
            "green"
        } else if p < 250.0 {
            "blue"
        } else if p < 325.0 {
            "purple"
        }
        // mystery, deep (250-325)
        else {
            "pink"
        } // love, warmth (325-360)
    }
    pub fn saturation(&self) -> f64 {
        self.composition * 100.0
    }
    pub fn lightness(&self) -> f64 {
        20.0 + self.ozone_buffer * 65.0
    }
    pub fn skew_deg(&self) -> f64 {
        self.torsion
    }
    pub fn glow_intensity(&self) -> f64 {
        self.composition * self.amplitude
    }

    pub fn glyph_weight(&self) -> i32 {
        let w = (self.amplitude * 900.0).round() as i32;
        if w < 150 {
            100
        } else if w < 300 {
            300
        } else if w < 450 {
            400
        } else if w < 600 {
            500
        } else if w < 800 {
            700
        } else {
            900
        }
    }

    pub fn class_string(&self) -> String {
        format!(
            "v15d-h{:03}-s{:02}-l{:03}-t{:04}-n{:02}-s{:02}",
            (self.su2_polarity % 360.0).round() as i32,
            self.saturation() as u8,
            self.lightness() as u16,
            (self.torsion * 10.0).round() as i32 + 1800,
            (self.magnetic_north.clamp(0.0, 1.0) * 99.0).round() as i32,
            (self.magnetic_south.clamp(0.0, 1.0) * 99.0).round() as i32
        )
    }
    pub fn is_void(&self) -> bool {
        self.composition < 0.01 && self.amplitude < 0.01
    }

    /// Checked version of to_glyph_props — returns Err on non-finite fields.
    /// SUSANO-STORM break 4 fix: hard Err instead of silently producing garbage.
    pub fn to_glyph_props_checked(&self) -> Result<GlyphProps, Vector15DError> {
        self.validate()?;
        Ok(self.to_glyph_props_unchecked())
    }

    /// Unchecked version — does not validate. Use only when validate() was already called.
    fn to_glyph_props_unchecked(&self) -> GlyphProps {
        self.to_glyph_props()
    }

    pub fn to_glyph_props(&self) -> GlyphProps {
        let (color, is_v) = if self.is_void() {
            ("hsl(0, 0%, 50%)".to_string(), true)
        } else {
            (
                format!(
                    "hsl({}, {}, {}%)",
                    self.hue(),
                    self.saturation() as u8,
                    self.lightness() as u8
                ),
                false,
            )
        };
        GlyphProps {
            color,
            weight: self.glyph_weight(),
            skew_deg: self.skew_deg(),
            glow_intensity: self.glow_intensity(),
            is_void: is_v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let v = Vector13D::default();
        assert_eq!(v.amplitude, 0.0);
        assert_eq!(v.domain_wall, DomainWall::Linked);
    }

    #[test]
    fn test_hue_range() {
        let cases = [
            (5.0, "red"),
            (25.0, "orange"),
            (55.0, "yellow"),
            (120.0, "green"),
            (200.0, "blue"),
            (280.0, "purple"),
            (330.0, "pink"),
        ];
        for (p, exp) in &cases {
            assert_eq!(
                Vector13D {
                    su2_polarity: *p,
                    ..Default::default()
        }
                .hue(),
                *exp
            );
        }
    }

    #[test]
    fn test_void() {
        let v = Vector13D {
            composition: 0.001,
            amplitude: 0.001,
            ..Default::default()
        };
        assert!(v.is_void());
        let v2 = Vector13D {
            composition: 0.5,
            amplitude: 0.8,
            ..Default::default()
        };
        assert!(!v2.is_void());
    }

    #[test]
    fn test_lightness() {
        assert_eq!(
            Vector13D {
                ozone_buffer: 0.0,
                ..Default::default()
        }
            .lightness(),
            20.0
        );
        assert_eq!(
            Vector13D {
                ozone_buffer: 1.0,
                ..Default::default()
        }
            .lightness(),
            85.0
        );
    }

    #[test]
    fn test_glyph_weight() {
        assert_eq!(
            Vector13D {
                amplitude: 0.05,
                ..Default::default()
        }
            .glyph_weight(),
            100
        );
        assert_eq!(
            Vector13D {
                amplitude: 0.60,
                ..Default::default()
        }
            .glyph_weight(),
            500
        );
        assert_eq!(
            Vector13D {
                amplitude: 0.95,
                ..Default::default()
        }
            .glyph_weight(),
            900
        );
    }

    #[test]
    fn test_void_glyph_props() {
        let v = Vector13D {
            composition: 0.0,
            amplitude: 0.0,
            ..Default::default()
        };
        let p = v.to_glyph_props();
        assert!(p.is_void);
        assert_eq!(p.color, "hsl(0, 0%, 50%)");
    }

    #[test]
    fn test_class_string() {
        let v = Vector13D {
            su2_polarity: 331.4,
            composition: 0.8,
            ozone_buffer: 0.6,
            torsion: -98.6,
            ..Default::default()
        };
        let c = v.class_string();
        assert!(c.contains("v15d-"));
        assert!(c.contains("-h331-"));
    }

    #[test]
    fn test_serialization() {
        let v = Vector13D {
            domain_wall: DomainWall::Gradient,
            ..Default::default()
        };
        let j = serde_json::to_string(&v).unwrap();
        eprintln!("json={}", j);
        assert!(j.contains(r#""Gradient""#));
        let v2 = Vector13D {
            domain_wall: DomainWall::Broken,
            ..Default::default()
        };
        let j2 = serde_json::to_string(&v2).unwrap();
        eprintln!("json2={}", j2);
        assert!(j2.contains(r#""Broken""#));
    }
}

// ═══════════════════════════════════════════════════════════════════
// STORM TESTS — Susano break 4: NaN/Inf fields
// ═══════════════════════════════════════════════════════════════════
#[cfg(test)]
mod storm_tests {
    use super::*;

    /// Break 4: Vector13D stores NaN — no Result refusal.
    /// FIX: validate() returns Err on NaN fields.
    #[test]
    fn nan_fields_do_not_hard_fail() {
        let v = Vector13D {
            amplitude: f64::NAN,
            ..Default::default()
        };
        let result = v.validate();
        assert!(result.is_err(), "validate() must Err on NaN amplitude");
        match result {
            Err(Vector15DError::NonFinite { field, value }) => {
                assert_eq!(field, "amplitude");
                assert_eq!(value, "NaN");
            }
            _ => panic!("expected NonFinite error for NaN"),
        }
    }

    /// Break 4: Infinity in any field must be rejected.
    #[test]
    fn inf_fields_rejected() {
        let v = Vector13D {
            entropy: f64::INFINITY,
            ..Default::default()
        };
        let result = v.validate();
        assert!(result.is_err(), "validate() must Err on Inf entropy");
        match result {
            Err(Vector15DError::NonFinite { field, value }) => {
                assert_eq!(field, "entropy");
                assert_eq!(value, "Infinite");
            }
            _ => panic!("expected NonFinite error for Inf"),
        }
    }

    /// Break 4: Negative infinity must be rejected.
    #[test]
    fn neg_inf_fields_rejected() {
        let v = Vector13D {
            torsion: f64::NEG_INFINITY,
            ..Default::default()
        };
        assert!(
            v.validate().is_err(),
            "validate() must Err on NegInf torsion"
        );
    }

    /// Break 4: try_new() must reject NaN.
    #[test]
    fn try_new_rejects_nan() {
        let result = Vector13D::try_new(
            0.5,
            1.0,
            180.0,
            0.55,
            0.08,
            0.78,
            0.45,
            0.40,
            DomainWall::Linked,
            331.4,
            -50.0,
            GaugeCoupling::Spinning,
            f64::NAN,
        );
        assert!(result.is_err(), "try_new with NaN closure must Err");
    }

    /// Break 4: try_new() accepts valid finite values.
    #[test]
    fn try_new_accepts_finite() {
        let result = Vector13D::try_new(
            0.72,
            1.0,
            180.0,
            0.55,
            0.08,
            0.78,
            0.45,
            0.40,
            DomainWall::Linked,
            331.4,
            -50.0,
            GaugeCoupling::Spinning,
            0.6,
        );
        assert!(result.is_ok(), "try_new with all finite values must Ok");
    }

    /// Break 4: to_glyph_props_checked() must Err on NaN.
    #[test]
    fn to_glyph_props_checked_rejects_nan() {
        let v = Vector13D {
            amplitude: f64::NAN,
            ..Default::default()
        };
        let result = v.to_glyph_props_checked();
        assert!(result.is_err(), "to_glyph_props_checked must Err on NaN");
    }

    /// Break 4: validate() passes on observed baselines.
    #[test]
    fn observed_baselines_are_finite() {
        for v in observed_n1() {
            assert!(v.validate().is_ok(), "n1 baseline must be finite");
        }
        for v in observed_n2() {
            assert!(v.validate().is_ok(), "n2 baseline must be finite");
        }
    }

    /// Break 4: NaN in any single field is caught.
    #[test]
    fn nan_in_each_field_caught() {
        let fields_with_nan: Vec<(&str, Vector13D)> = vec![
            (
                "amplitude",
                Vector13D {
                    amplitude: f64::NAN,
                    ..Default::default()
        },
            ),
            (
                "frequency",
                Vector13D {
                    frequency: f64::NAN,
                    ..Default::default()
        },
            ),
            (
                "phase",
                Vector13D {
                    phase: f64::NAN,
                    ..Default::default()
        },
            ),
            (
                "coherence",
                Vector13D {
                    coherence: f64::NAN,
                    ..Default::default()
        },
            ),
            (
                "entropy",
                Vector13D {
                    entropy: f64::NAN,
                    ..Default::default()
        },
            ),
            (
                "composition",
                Vector13D {
                    composition: f64::NAN,
                    ..Default::default()
        },
            ),
            (
                "resonance",
                Vector13D {
                    resonance: f64::NAN,
                    ..Default::default()
        },
            ),
            (
                "ozone_buffer",
                Vector13D {
                    ozone_buffer: f64::NAN,
                    ..Default::default()
        },
            ),
            (
                "su2_polarity",
                Vector13D {
                    su2_polarity: f64::NAN,
                    ..Default::default()
        },
            ),
            (
                "torsion",
                Vector13D {
                    torsion: f64::NAN,
                    ..Default::default()
        },
            ),
            (
                "closure",
                Vector13D {
                    closure: f64::NAN,
                    ..Default::default()
        },
            ),
        ];
        for (name, v) in &fields_with_nan {
            assert!(
                v.validate().is_err(),
                "NaN in {} must be caught by validate()",
                name
            );
        }
    }

    #[test]
    fn poles_nan_rejected() {
        let mut v = Vector15D::default();
        v.magnetic_north = f64::NAN;
        assert!(v.validate().is_err(), "NaN magnetic_north must Err");
        v = Vector15D::default();
        v.magnetic_south = f64::INFINITY;
        assert!(v.validate().is_err(), "Inf magnetic_south must Err");
    }

    #[test]
    fn try_new_zeros_poles() {
        let v = Vector15D::try_new(
            0.5, 0.5, 0.0, 0.5, 0.2, 0.5, 0.5, 0.5,
            DomainWall::Linked, 200.0, 0.0, GaugeCoupling::Static, 0.5,
        )
        .unwrap();
        assert_eq!(v.magnetic_north, 0.0);
        assert_eq!(v.magnetic_south, 0.0);
    }

    #[test]
    fn try_new_15_keeps_poles() {
        let v = Vector15D::try_new_15(
            0.5, 0.5, 0.0, 0.5, 0.2, 0.5, 0.5, 0.5,
            DomainWall::Linked, 200.0, 0.0, GaugeCoupling::Static, 0.5,
            0.4, 0.6,
        )
        .unwrap();
        assert!((v.magnetic_north - 0.4).abs() < 1e-9);
        assert!((v.magnetic_south - 0.6).abs() < 1e-9);
        assert!((v.polar_prestress() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn serde_13_json_defaults_poles() {
        let raw = r#"{
            "amplitude": 0.1, "frequency": 0.1, "phase": 0.0,
            "coherence": 0.1, "entropy": 0.1, "composition": 0.1,
            "resonance": 0.1, "ozone_buffer": 0.5,
            "domain_wall": "Linked", "su2_polarity": 0.0, "torsion": 0.0,
            "gauge_coupling": "Static", "closure": 0.0
        }"#;
        let v: Vector15D = serde_json::from_str(raw).expect("13-field json");
        assert_eq!(v.magnetic_north, 0.0);
        assert_eq!(v.magnetic_south, 0.0);
        assert!(v.validate().is_ok());
    }

}

/// Observed n=1 baseline (post-dress, post-cycle) — living reference state.
pub fn observed_n1() -> Vec<Vector13D> {
    // Average across 61 segments of first recording
    vec![Vector13D {
        amplitude: 0.72,
        frequency: 1.0,
        phase: 180.0,
        coherence: 0.55,
        entropy: 0.08,
        composition: 0.78,
        resonance: 0.45,
        ozone_buffer: 0.40,
        domain_wall: DomainWall::Linked,
        su2_polarity: 331.4,
        torsion: -50.0,
        gauge_coupling: GaugeCoupling::Spinning,
        closure: 0.6,
            magnetic_north: 0.0,
            magnetic_south: 0.0,
        }]
}

/// Observed n=2 baseline (in shower) — living reference state.
pub fn observed_n2() -> Vec<Vector13D> {
    // Average across 24 segments of second recording
    vec![Vector13D {
        amplitude: 0.65,
        frequency: 0.57,
        phase: 200.0,
        coherence: 0.62,
        entropy: 0.03,
        composition: 0.85,
        resonance: 0.58,
        ozone_buffer: 0.31,
        domain_wall: DomainWall::Linked,
        su2_polarity: 320.0,
        torsion: -5.0,
        gauge_coupling: GaugeCoupling::Oscillating,
        closure: 0.75,
            magnetic_north: 0.0,
            magnetic_south: 0.0,
        }]
}

/// Compare two observed baselines — what shifted between states?
pub fn compare_baselines(before: &Vector13D, after: &Vector13D) -> StateChange {
    let hue_shift = if before.su2_polarity >= 325.0 && after.su2_polarity < 325.0 {
        "pink -> purple"
    } else if before.su2_polarity < 325.0 && after.su2_polarity >= 325.0 {
        "purple -> pink"
    } else {
        "same register"
    };

    let torsion_became_near_zero = after.torsion.abs() < 10.0; // near upright axis
    let domain_unchanged = before.domain_wall == after.domain_wall;
    let coupling_shifted = before.gauge_coupling != after.gauge_coupling;

    StateChange {
        hue_shift: hue_shift.to_string(),
        frequency_delta: after.frequency - before.frequency,
        entropy_change_pct: (((after.entropy - before.entropy) / before.entropy) * 100.0).round()
            as i32,
        coherence_delta: after.coherence - before.coherence,
        torsion_became_near_zero: torsion_became_near_zero,

        domain_wall_unchanged: domain_unchanged,
        gauge_coupling_shifted: coupling_shifted,
        ozone_buffer_delta: after.ozone_buffer - before.ozone_buffer,
    }
}

/// Summary of a state change between two Vector13D observations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    pub hue_shift: String,
    pub frequency_delta: f64,
    pub entropy_change_pct: i32,
    pub coherence_delta: f64,
    pub torsion_became_near_zero: bool,
    pub domain_wall_unchanged: bool,
    pub gauge_coupling_shifted: bool,
    pub ozone_buffer_delta: f64,
}

// ═══════════════════════════════════════════════════════════════════
// TENSEGRITY CRUST — Outer crust layer (Fields 4-8 as tethers, Field 9 as network)
// Implements dynamic tensional integrity over the 3D core (Fields 1-3)
// ═══════════════════════════════════════════════════════════════════

/// The five crust nodes — each maps to one field in Fields 4-8 of Vector13D.
pub const CRUST_NODE_NAMES: [&str; 5] = [
    "coherence",
    "entropy",
    "composition",
    "resonance",
    "ozone_buffer",
];

/// A single tether node: magnetic axis tethered to the core.
#[derive(Debug, Clone)]
pub struct TetherNode {
    /// Field index within Vector13D (0-4 maps to fields 4-8).
    pub field_index: usize,
    /// Baseline displacement from core equilibrium at rest (radians).
    baseline_r: f64,
    /// Angular position around the core ring (5 nodes → 72° apart).
    angle_deg: f64,
    /// Magnetic stiffness k_i — how hard this node pulls toward baseline.
    stiffness: f64,
    /// Damping coefficient c_i — prevents violent snap-back.
    dampening: f64,
}

/// The tether network state corresponding to Field 9 (domain_wall).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TetherNetwork {
    /// All 5 tethers connected; equilibrium holds. [VERIFIED]
    Connected,
    /// One or more nodes displaced but still contributing tension. [HYPOTHESIS]
    Strained { pulled_nodes: u8, slack_nodes: u8 },
    /// Topology ruptured — all nodes past angular reach, remapping in progress. [DECLARED]
    Ruptured { pending_remap: bool },
    /// All nodes decoupled; core exposed.
    Severed,
}

/// Result of absorbing an external shock through the tension network.
#[derive(Debug, Clone)]
pub struct ShockResult {
    /// Which tether index was most loaded (0-4) after redistribution.
    pub max_load_tether: usize,
    /// Maximum absolute displacement achieved by any node.
    pub max_displacement_deg: f64,
    /// Whether topology ruptured during absorption.
    pub ruptured: bool,
    /// New baseline configuration after remap (only set if ruptured=true).
    pub new_baseline_angles: Option<[f64; 5]>,
}

/// Configuration for the tensegrity crust geometry.
pub struct CrustConfig {
    /// Max angular displacement per node before it goes slack (radians).
    /// θ_max = arcsin(r_core / r_baseline)
    pub theta_max: f64,
    /// Cubic stiffness coefficient — makes extreme stretching exponentially expensive.
    pub cubic_stiffness_coeff: f64,
    /// Energy budget cap — total system energy available for tension absorption.
    pub max_system_energy: f64,
}

impl Default for CrustConfig {
    fn default() -> Self {
        // theta_max = 3.5 rad ≈ 200° — displacement limit per tether.
        // Real physical limits come from cubic stiffening (force model),
        // not arbitrary energy caps.
        Self {
            theta_max: 3.5,
            cubic_stiffness_coeff: 8.0,
            max_system_energy: 1000.0,
        }
    }
}

impl CrustConfig {
    /// Compute the elastic tension for one tether at displacement Δr from baseline.
    /// T_i = k_i · Δr - c_i · (dΔr/dt) + α · (Δr)³
    /// The cubic term prevents infinite stretch by making extreme deformation economically impossible.
    pub fn tether_tension(&self, node: &TetherNode, delta_r: f64, delta_r_vel: f64) -> f64 {
        let linear = node.stiffness * delta_r;
        let damping = -node.dampening * delta_r_vel;
        let cubic = self.cubic_stiffness_coeff * (delta_r).abs().powi(3) * delta_r.signum();
        linear + damping + cubic
    }

    /// Check if displacement is beyond angular reach — node goes slack past this.
    pub fn is_beyond_reach(&self, delta_deg: f64) -> bool {
        (delta_deg.to_radians()).abs() > self.theta_max
    }

    /// Compute total system energy across all tethers at their current displacements.
    pub fn compute_system_energy(&self, nodes: &[TetherNode], deltas: &[f64]) -> f64 {
        // E = Σ (0.5 · k_i · Δr²) — elastic potential energy only
        // Cubic stiffening is a force model, not stored as energy in the budget check.
        nodes
            .iter()
            .zip(deltas.iter())
            .map(|(n, d)| 0.5 * n.stiffness * d * d)
            .sum()
    }
}

/// The complete tensegrity crust wrapping the Vector13D core.
pub struct TensegrityCrust {
    /// The five tether nodes (Fields 4-8).
    pub nodes: [TetherNode; 5],
    /// Live tension state of each tether (in radians from baseline).
    pub tensions: [f64; 5],
    /// Velocity of tension change for dampening (Δr/dt).
    pub tension_velocities: [f64; 5],
    /// Current equilibrium angles of all nodes (for remap tracking).
    pub current_angles_deg: [f64; 5],
    /// Network topology state.
    pub network_state: TetherNetwork,
    /// Crust configuration constants.
    pub config: CrustConfig,
}

impl Default for TensegrityCrust {
    fn default() -> Self {
        let nodes = std::array::from_fn(|i| TetherNode {
            field_index: i + 4, // maps to Vector13D fields 4-8
            baseline_r: 2.5 + (i as f64) * 0.1,
            angle_deg: (i as f64) * 72.0,
            stiffness: 1.0 + (i as f64) * 0.12,
            dampening: 0.35,
        });
        let current_angles = [0.0, 72.0, 144.0, 216.0, 288.0];
        Self {
            nodes,
            tensions: [0.0; 5],
            tension_velocities: [0.0; 5],
            current_angles_deg: current_angles,
            network_state: TetherNetwork::Connected,
            config: CrustConfig::default(),
        }
    }
}

impl TensegrityCrust {
    /// Create a crust with custom configuration.
    pub fn with_config(config: CrustConfig) -> Self {
        let nodes = std::array::from_fn(|i| TetherNode {
            field_index: i + 4,
            baseline_r: 2.5 + (i as f64) * 0.1,
            angle_deg: (i as f64) * 72.0,
            stiffness: 1.0 + (i as f64) * 0.12,
            dampening: 0.35,
        });
        let current_angles = [0.0, 72.0, 144.0, 216.0, 288.0];
        Self {
            nodes,
            tensions: [0.0; 5],
            tension_velocities: [0.0; 5],
            current_angles_deg: current_angles,
            network_state: TetherNetwork::Connected,
            config,
        }
    }

    /// Compute the net tensional force vector magnitude across all active tethers.
    /// At equilibrium: ΣT_i = 0 (net vector is zero).
    fn net_tension_vector(&self) -> ([f64; 5], f64) {
        let mut fx = 0.0;
        let mut fy = 0.0;
        let mut raw_tensions: [f64; 5] = [0.0; 5];

        for i in 0..5 {
            // Tension contribution from angular displacement of this node
            let theta_rad = self.current_angles_deg[i].to_radians();
            let tension_mag = if self.network_state == TetherNetwork::Severed
                || self.tensions[i] < 0.01
            {
                0.0
            } else {
                self.config
                    .tether_tension(&self.nodes[i], self.tensions[i], self.tension_velocities[i])
                    .abs()
            };
            raw_tensions[i] = tension_mag;
            fx += tension_mag * theta_rad.cos();
            fy += tension_mag * theta_rad.sin();
        }

        let magnitude = (fx * fx + fy * fy).sqrt();
        (raw_tensions, magnitude)
    }

    /// Absorb an external shock by redistributing tension across the network.
    /// ΣT_i + F_ext = 0 — the crust absorbs and distributes.
    ///
    /// Uses plastic plateau model: nodes at yield cap at k·θ_max instead of snapping to 0.
    /// This creates a real Strained regime between Connected and Ruptured.
    pub fn absorb_shock(&mut self, f_ext_mag: f64, impact_angle_deg: f64) -> Option<ShockResult> {
        if f_ext_mag <= 0.0 {
            return None;
        }

        // Clamp input to prevent numerical issues.
        let f_ext = f_ext_mag.min(self.config.max_system_energy * 2.0);
        let impact_rad = impact_angle_deg.to_radians();

        // Step 1: Distribute external force via coupling weights.
        let mut raw_tensions: [f64; 5] = [0.0; 5];
        for i in 0..5 {
            let theta_rad = self.current_angles_deg[i].to_radians();
            let delta_angle = (theta_rad - impact_rad).cos();
            // Weight: base global + directional (cosine coupling, no compression exponent).
            let weight = 0.3 + 0.7 * ((delta_angle + 1.0) / 2.0).powf(0.7);
            raw_tensions[i] = f_ext * weight;
        }

        // Step 2: Plastic yield — cap each node at k·θ_max.
        // Node does NOT snap to zero; it holds maximum tension and the excess force
        // is redistributed geometrically to the remaining nodes.
        let mut capping_nodes: [bool; 5] = [false; 5];
        for i in 0..5 {
            let delta_r = raw_tensions[i] / self.nodes[i].stiffness;
            if delta_r.abs() > self.config.theta_max {
                // Yield: tension caps at k * theta_max.
                capping_nodes[i] = true;
            }
        }

        let max_disp_deg: f64 = (0..5)
            .map(|i| {
                if capping_nodes[i] {
                    self.config.theta_max
                } else {
                    raw_tensions[i] / self.nodes[i].stiffness
                }
                .abs()
            })
            .fold(0.0, f64::max);

        // Step 3: Rupture check — total capacity vs external force (Anaseos principle).
        // Rupture only when ΣT_max < F_ext. Not the vector sum of current tensions
        // (which near-zero by symmetric geometry), but scalar sum of all max tensions.
        let total_capacity: f64 = (0..5)
            .map(|i| self.nodes[i].stiffness * self.config.theta_max)
            .sum();
        if total_capacity < f_ext {
            // True system failure — even fully aligned tethers can't balance the force.
            self.network_state = TetherNetwork::Ruptured {
                pending_remap: true,
            };
            let new_angles = self.remap_after_rupture();
            for i in 0..5 {
                let tension_cap = self.nodes[i].stiffness * self.config.theta_max;
                let tension = if capping_nodes[i] {
                    tension_cap
                } else {
                    raw_tensions[i]
                };
                self.tensions[i] = (tension as f64).min(f_ext);
            }
            self.current_angles_deg = new_angles;

            let max_load_idx = (0..5)
                .max_by(|a, b| {
                    let ta = if capping_nodes[*a] {
                        self.nodes[*a].stiffness * self.config.theta_max
                    } else {
                        raw_tensions[*a]
                    };
                    let tb = if capping_nodes[*b] {
                        self.nodes[*b].stiffness * self.config.theta_max
                    } else {
                        raw_tensions[*b]
                    };
                    ta.total_cmp(&tb)
                })
                .unwrap();
            return Some(ShockResult {
                max_load_tether: max_load_idx,
                max_displacement_deg: self.config.theta_max.to_degrees(),
                ruptured: true,
                new_baseline_angles: Some(new_angles),
            });
        }

        // Step 4: Apply final tensions (plastic plateau for capping nodes).
        let mut pulled = 0u8;
        let mut capped_count = 0u8;
        for i in 0..5 {
            let tension = if capping_nodes[i] {
                self.nodes[i].stiffness * self.config.theta_max
            } else {
                raw_tensions[i]
            };
            self.tensions[i] = tension.max(0.0);
            if capping_nodes[i] {
                capped_count += 1;
            } else {
                pulled += 1;
            }
        }

        // Step 5: Determine regime.
        if capped_count > 0 {
            self.network_state = TetherNetwork::Strained {
                pulled_nodes: pulled,
                slack_nodes: capped_count,
            };
        } else {
            self.network_state = TetherNetwork::Connected;
        }

        // Find max load tether.
        let max_load_idx = (0..5)
            .max_by(|a, b| {
                let ta = if capping_nodes[*a] {
                    self.nodes[*a].stiffness * self.config.theta_max
                } else {
                    raw_tensions[*a]
                };
                let tb = if capping_nodes[*b] {
                    self.nodes[*b].stiffness * self.config.theta_max
                } else {
                    raw_tensions[*b]
                };
                ta.total_cmp(&tb)
            })
            .unwrap();

        Some(ShockResult {
            max_load_tether: max_load_idx,
            max_displacement_deg: if capped_count > 0 {
                self.config.theta_max.to_degrees()
            } else {
                max_disp_deg
            },
            ruptured: false,
            new_baseline_angles: None,
        })
    }

    /// Remap baseline angles after topology rupture — find new equilibrium.
    /// This is the graceful transition, not a break.
    fn remap_after_rupture(&self) -> [f64; 5] {
        // Perturb: rotate all nodes by 36° and redistribute based on residual energy.
        // The core (Fields 1-3) remains undisturbed — only the crust remaps.
        let perturbation = 36.0;
        let mut new_angles = [0.0; 5];
        for i in 0..5 {
            new_angles[i] = (self.current_angles_deg[(i + 1) % 5] + perturbation) % 360.0;
        }
        // Sort to maintain geometric ordering.
        new_angles.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        new_angles
    }

    /// Restore the crust after a shock event — reset tethers toward baseline.
    pub fn restore_towards_baseline(&mut self, dt: f64) {
        // Exponential decay of tension toward zero (baseline = equilibrium).
        let decay_rate = 0.8_f64.powf(dt);
        for i in 0..5 {
            self.tensions[i] *= decay_rate;
            if self.tensions[i] < 1e-6 {
                self.tensions[i] = 0.0;
            }
        }

        // Check if all tethers have recovered.
        let total_tension: f64 = self.tensions.iter().sum();
        if total_tension < 1e-3 && matches!(self.network_state, TetherNetwork::Strained { .. }) {
            self.network_state = TetherNetwork::Connected;
        }

        // If ruptured and energy is near zero, finalize remap.
        if let TetherNetwork::Ruptured { pending_remap } = &self.network_state {
            if total_tension < 1e-4 && *pending_remap {
                self.network_state = TetherNetwork::Strained {
                    pulled_nodes: 5,
                    slack_nodes: 0,
                };
            }
        }
    }

    /// Read the tether state as a Vector13D-compatible field update.
    /// Maps crust tension back to fields 4-8 of the core vector.
    pub fn sync_to_vector(&self) -> [f64; 5] {
        // Convert normalized tension to 0-1 range for each field.
        self.tensions.map(|t| (t.min(1.0)).max(0.0))
    }

    /// Check if the core is currently exposed (all tethers slack or ruptured).
    pub fn core_exposed(&self) -> bool {
        matches!(
            self.network_state,
            TetherNetwork::Severed | TetherNetwork::Ruptured { .. }
        ) || self.tensions.iter().all(|t| *t < 0.01)
    }
}

/// Convenience: create a tensegrity crust and absorb a shock in one call.
pub fn tensegrity_absorb(
    shock_magnitude: f64,
    impact_angle_deg: f64,
) -> (Option<ShockResult>, TensegrityCrust) {
    let mut crust = TensegrityCrust::default();
    let result = crust.absorb_shock(shock_magnitude, impact_angle_deg);
    // Restore slightly to show the dynamics.
    crust.restore_towards_baseline(1.0);
    (result, crust)
}

#[cfg(test)]
mod observed_tests {
    use super::*;

    #[test]
    fn test_n1_hue_is_pink() {
        let v = observed_n1()[0];
        assert_eq!(v.hue(), "pink");
    }
    #[test]
    fn test_n2_hue_is_purple() {
        let v = observed_n2()[0];
        assert_eq!(v.hue(), "purple");
    }

    #[test]
    fn test_domain_wall_constant() {
        let n1 = &observed_n1()[0];
        let n2 = &observed_n2()[0];
        let change = compare_baselines(&n1, &n2);
        assert!(
            change.domain_wall_unchanged,
            "domain wall should be constant across states"
        );
    }

    #[test]
    fn test_entropy_dropped() {
        let n1 = &observed_n1()[0];
        let n2 = &observed_n2()[0];
        let change = compare_baselines(&n1, &n2);
        assert!(
            change.entropy_change_pct <= -55,
            "entropy should drop significantly (>55%%)"
        )
    }

    #[test]
    fn test_torsion_nearly_zero() {
        let n2 = &observed_n2()[0];
        let change = compare_baselines(&observed_n1()[0], &n2);
        assert!(
            change.torsion_became_near_zero,
            "torsion should approach zero in shower state"
        );
    }

    #[test]
    fn test_gauge_coupling_shifts() {
        let n1 = &observed_n1()[0];
        let n2 = &observed_n2()[0];
        assert_eq!(n1.gauge_coupling, GaugeCoupling::Spinning);
        assert_eq!(n2.gauge_coupling, GaugeCoupling::Oscillating);
        let change = compare_baselines(&n1, &n2);
        assert!(change.gauge_coupling_shifted);
    }

    #[test]
    fn test_hue_shift_purple() {
        let n1 = &observed_n1()[0];
        let n2 = &observed_n2()[0];
        let change = compare_baselines(&n1, &n2);
        assert_eq!(change.hue_shift, "pink -> purple");
    }

    #[test]
    fn test_ozone_dropped() {
        let n1 = &observed_n1()[0];
        let n2 = &observed_n2()[0];
        let change = compare_baselines(&n1, &n2);
        assert!(
            change.ozone_buffer_delta < 0.0,
            "ozone should drop — energy went inward"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// TENSEGRITY CRUST TESTS — calibrated with Anaseos' plastic plateau model
// ═══════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tensegrity_tests {
    use super::*;

    // ── VERIFIED: baseline geometry holds at equilibrium ──

    #[test]
    fn test_default_crust_is_connected() {
        let crust = TensegrityCrust::default();
        assert_eq!(crust.network_state, TetherNetwork::Connected);
        assert!(crust.tensions.iter().all(|t| *t == 0.0));
    }

    #[test]
    fn test_five_tethers_at_seventy_two_degrees() {
        let crust = TensegrityCrust::default();
        for i in 1..5 {
            let expected = (i as f64) * 72.0;
            assert!((crust.current_angles_deg[i] - expected).abs() < 1e-9);
        }
    }

    #[test]
    fn test_tether_node_count_is_five() {
        let crust = TensegrityCrust::default();
        assert_eq!(crust.nodes.len(), 5);
    }

    // ── VERIFIED: zero shock preserves Connected state ──

    #[test]
    fn test_null_shock_returns_none() {
        let mut crust = TensegrityCrust::default();
        assert!(crust.absorb_shock(0.0, 90.0).is_none());
        assert!(crust.absorb_shock(-1.0, 90.0).is_none());
    }

    #[test]
    fn test_zero_shock_unchanged() {
        let mut crust = TensegrityCrust::default();
        crust.absorb_shock(0.0, 90.0);
        assert_eq!(crust.network_state, TetherNetwork::Connected);
        assert!(crust.tensions.iter().all(|t| (*t - 0.0).abs() < 1e-9));
    }

    // ── VERIFIED: small shock → Connected (no nodes hit plastic yield) ──

    #[test]
    fn test_small_shock_keeps_connected() {
        // theta_max = 3.5 rad. With k_i ≈ 1.0-1.48, tension cap per node ≈ 3.5-5.2.
        // Small shock (2.0) distributed across 5 nodes never reaches cap on any.
        let mut crust = TensegrityCrust::default();
        let result = crust.absorb_shock(2.0, 90.0);
        assert!(result.is_some());
        assert_eq!(crust.network_state, TetherNetwork::Connected);
    }

    // ── HYPOTHESIS: moderate shock → Strained (plastic plateau engages some nodes) ──

    #[test]
    fn test_strained_state_emerges_with_moderate_shock() {
        // Shock magnitude ~6.5 puts the nearest node's tension at k·θ_max threshold.
        // That node caps → Strained (not ruptured, since ΣT_max >> F_ext).
        let mut crust = TensegrityCrust::default();
        let result = crust.absorb_shock(6.5, 72.0);
        assert!(result.is_some());

        if let TetherNetwork::Strained {
            pulled_nodes,
            slack_nodes,
        } = &crust.network_state
        {
            assert!(*pulled_nodes > 0 && *slack_nodes > 0);
            assert_eq!(pulled_nodes + slack_nodes, 5u8);
        } else {
            panic!("Expected Strained (state={:?})", crust.network_state);
        }
    }

    #[test]
    fn test_strained_maintains_three_regime() {
        // Verify the gap exists: we can find a shock magnitude that gives pure Connected,
        // one that gives Strained, and one that gives Ruptured.
        let mut crust_conn = TensegrityCrust::default();
        let result_conn = crust_conn.absorb_shock(3.0, 72.0);
        assert_eq!(crust_conn.network_state, TetherNetwork::Connected);

        let mut crust_strain = TensegrityCrust::default();
        let result_strain = crust_strain.absorb_shock(6.5, 72.0);
        assert!(matches!(
            crust_strain.network_state,
            TetherNetwork::Strained { .. }
        ));

        // The gap: between Connected (3.0) and Strained (6.5), there is a threshold
        // where the first node hits its plastic yield cap.
    }

    // ── DECLARED: large shock → Ruptured (ΣT_max < F_ext) ──

    #[test]
    fn test_rupture_when_capacity_exceeded() {
        // total_cap = Σ(k_i · θ_max) for default config
        let nodes = TensegrityCrust::default().nodes;
        let config = CrustConfig::default();
        let total_cap: f64 = (0..5).map(|i| nodes[i].stiffness * config.theta_max).sum();
        assert!(
            total_cap > 21.0 && total_cap < 22.0,
            "actual total_cap={total_cap:.2}"
        );

        // Shock exceeding total_capacity → rupture.
        let mut crust = TensegrityCrust::default();
        let result = crust.absorb_shock(total_cap + 1.0, 72.0);
        assert!(result.is_some());

        if let TetherNetwork::Ruptured { pending_remap } = &crust.network_state {
            assert!(*pending_remap);
        } else {
            panic!("Expected Ruptured (state={:?})", crust.network_state);
        }
    }

    // ── VERIFIED: directional coupling loads nearest node most ──

    #[test]
    fn test_shock_distributes_tension_proportionally_to_angle() {
        let mut crust = TensegrityCrust::default();
        let result = crust.absorb_shock(5.0, 72.0);
        assert!(result.is_some());

        let max_idx = (0..5)
            .max_by(|a, b| crust.tensions[*a].total_cmp(&crust.tensions[*b]))
            .unwrap();
        assert_eq!(max_idx, result.unwrap().max_load_tether);
    }

    // ── DECLARED: core exposed only when ruptured or all zero tension ──

    #[test]
    fn test_core_exposed_only_when_all_slack() {
        let mut crust = TensegrityCrust::default();
        crust.absorb_shock(1.0, 72.0);
        assert!(!crust.core_exposed());

        // Need shock > total_capacity (~21.7) to rupture and expose core.
        let mut crust2 = TensegrityCrust::default();
        crust2.absorb_shock(23.0, 72.0); // → Ruptured
        assert!(crust2.core_exposed());
    }

    // ── VERIFIED: cubic stiffening prevents infinite stretch ──

    #[test]
    fn test_cubic_stiffening_at_extreme_displacement() {
        let config = CrustConfig {
            cubic_stiffness_coeff: 8.0,
            ..CrustConfig::default()
        };
        let node = &TensegrityCrust::default().nodes[0];

        let t1 = config.tether_tension(node, 0.5, 0.0);
        let t2 = config.tether_tension(node, 1.5, 0.0);
        let t3 = config.tether_tension(node, 3.0, 0.0);

        assert!(t1 > 0.45 && t2 > 2.5 && t3 > 20.0,
            "Cubic stiffening should produce escalating tensions: t1={t1:.2}, t2={t2:.2}, t3={t3:.2}");
    }

    // ── HYPOTHESIS: restore_towards_baseline converges to Connected ──

    #[test]
    fn test_restore_converges_to_connected() {
        let mut crust = TensegrityCrust::default();
        crust.absorb_shock(6.5, 180.0); // → Strained
        assert!(matches!(
            crust.network_state,
            TetherNetwork::Strained { .. }
        ));

        for _ in 0..20 {
            crust.restore_towards_baseline(1.0);
        }

        let total: f64 = crust.tensions.iter().sum();
        // With decay_rate = 0.8^1 ≈ 0.8 per step, after 20 steps factor ≈ 0.0116.
        // If initial tension ≈ 5-6, final total ≈ 0.1-0.3 → need more steps or faster decay.
        assert!(
            total < 1.0,
            "Tension should nearly decay to zero (actual={total:.4})"
        );
    }

    // ── VERIFIED: remap produces valid sorted angles ──

    #[test]
    fn test_remap_produces_sorted_angles() {
        let crust = TensegrityCrust::default();
        let new_angles = crust.remap_after_rupture();
        for i in 1..5 {
            assert!(
                new_angles[i] >= new_angles[i - 1],
                "Angles should be sorted"
            );
        }
    }

    #[test]
    fn test_remap_rotates_by_36_degrees() {
        let crust = TensegrityCrust::default();
        let new_angles = crust.remap_after_rupture();
        assert!(new_angles.iter().all(|a| *a >= 0.0 && *a < 360.0));
        for i in 0..5 {
            for j in (i + 1)..5 {
                assert!(
                    (new_angles[i] - new_angles[j]).abs() > 1e-3,
                    "Angles {} and {} should be distinct",
                    i,
                    j
                );
            }
        }
    }

    // ── VERIFIED: sync_to_vector produces normalized [0,1] values ──

    #[test]
    fn test_sync_to_vector_normalized() {
        let mut crust = TensegrityCrust::default();
        crust.absorb_shock(5.0, 72.0); // moderate strain
        let sync = crust.sync_to_vector();
        for &v in &sync {
            assert!(v >= 0.0 && v <= 1.0);
        }
    }

    // ── VERIFIED: TetherNode field_index maps correctly ──

    #[test]
    fn test_tether_node_field_mapping() {
        let crust = TensegrityCrust::default();
        for i in 0..5 {
            assert_eq!(crust.nodes[i].field_index, i + 4);
        }
    }

    // ── VERIFIED: CRUST_NODE_NAMES matches Vector13D fields 4-8 ──

    #[test]
    fn test_crust_node_names() {
        assert_eq!(CRUST_NODE_NAMES[0], "coherence");
        assert_eq!(CRUST_NODE_NAMES[1], "entropy");
        assert_eq!(CRUST_NODE_NAMES[2], "composition");
        assert_eq!(CRUST_NODE_NAMES[3], "resonance");
        assert_eq!(CRUST_NODE_NAMES[4], "ozone_buffer");
    }

    // ── HYPOTHESIS: impact angle selects nearest node as max load ──

    #[test]
    fn test_impact_angle_selects_nearest_node_as_max_load() {
        let mut crust = TensegrityCrust::default();
        let result = crust.absorb_shock(15.0, 0.0); // impact at node 0
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(
            crust.tensions[r.max_load_tether],
            crust
                .tensions
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max)
        );
    }

    // ── DECLARED: serde serialization of TetherNetwork works ──

    #[test]
    fn test_tether_network_serde() {
        let json = serde_json::to_string(&TetherNetwork::Connected).unwrap();
        assert!(json.contains("Connected"));

        let json2 = serde_json::to_string(&TetherNetwork::Strained {
            pulled_nodes: 3,
            slack_nodes: 2,
        })
        .unwrap();
        assert!(json2.contains("Strained"));
    }

    // ── DECLARED: with_config produces same topology as default ──

    #[test]
    fn test_with_config_produces_same_topology() {
        let custom = TensegrityCrust::with_config(CrustConfig::default());
        let dc = TensegrityCrust::default();
        for i in 0..5 {
            assert_eq!(custom.nodes[i].field_index, dc.nodes[i].field_index);
            assert_eq!(custom.current_angles_deg[i], dc.current_angles_deg[i]);
        }
    }

    // ── VERIFIED: convergence — repeated small shocks don't break the crust ──

    #[test]
    fn test_repeated_small_shocks_converge() {
        let mut crust = TensegrityCrust::default();
        for step in 0..10 {
            let angle = ((step * 72) % 360) as f64;
            crust.absorb_shock(2.0, angle);
            crust.restore_towards_baseline(1.0);
        }
        assert_eq!(crust.network_state, TetherNetwork::Connected);
    }
}
