//! DECLARED astronomical profiles — envelope context, never geometry.
//!
//! Per-body axial geometry, ring inventory, and satellite census for the
//! bodies the pantheon seats reference. These are *calibration-shaped*
//! context: they describe how a seat's observations sit in the real solar
//! system. Nothing here is a codec field; `Vector15D` stays canonical.
//!
//! Every value carries its epistemic status via [`Measurement`] — plain
//! `Option` conflates "absent", "unknown", "bounded" and "not applicable",
//! and that conflation is exactly the failure class DECLARED discipline
//! exists to prevent. Counts that move carry a mandatory `census_epoch`
//! (exact ISO date where an announcement exists); undated census data goes
//! stale silently. See docs/HUBBLE_MIRROR.md and docs/BODY_PROFILES.md.

use serde::{Deserialize, Serialize};

/// A value plus its epistemic status. The distinction between `Absent`,
/// `Unknown`, `UpperBound`/`LowerBound` and `NotApplicable` is load-bearing:
/// Venus has no detected dipole (Absent), Pluto's field is unmeasured
/// (Unknown), Saturn's tilt is an upper bound (UpperBound) — none of these
/// is a number, and none of them is the same claim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Measurement {
    /// A reported value. `epoch`/`model` required where the quantity is
    /// time-varying or model-derived (e.g. JRM33 for Jupiter's field).
    Measured {
        value: f64,
        epoch: Option<String>,
        model: Option<String>,
    },
    /// Only an upper limit is known (Saturn's tilt < 0.007 deg).
    UpperBound { limit: f64 },
    /// Only a lower limit is known.
    LowerBound { limit: f64 },
    /// Observationally unsupported — e.g. no detected intrinsic dipole.
    Absent,
    /// Applicable but not adequately measured.
    Unknown,
    /// The field has no physical meaning for this profile.
    NotApplicable,
}

impl Measurement {
    pub fn measured(value: f64) -> Self {
        Self::Measured { value, epoch: None, model: None }
    }

    pub fn measured_model(value: f64, model: &str) -> Self {
        Self::Measured {
            value,
            epoch: None,
            model: Some(model.into()),
        }
    }

    /// The numeric value where one exists (Measured or a Bound limit).
    /// Absent/Unknown/NotApplicable yield None — there is no number.
    pub fn value(&self) -> Option<f64> {
        match self {
            Self::Measured { value, .. } => Some(*value),
            Self::UpperBound { limit } | Self::LowerBound { limit } => Some(*limit),
            _ => None,
        }
    }
}

/// A solar-system body a profile can describe. Seat binding is external —
/// the codec ships bodies; the pantheon binds seats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Body {
    Sun,
    Mercury,
    Venus,
    Earth,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Pluto,
    Haumea,
}

/// Rotational and magnetic axial geometry.
/// `obliquity_deg` uses the 0-180 positive-pole convention (>90 =
/// retrograde); note some NASA tables report the acute angle between the
/// spin axis and the orbital plane instead (Pluto ~57 vs ~120).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AxialProfile {
    pub obliquity_deg: Measurement,
    /// Dipole axis tilt vs rotation axis. Venus/Mars: Absent (no intrinsic
    /// global field). Pluto/Haumea: Unknown.
    pub magnetic_tilt_deg: Measurement,
    /// Dipole displacement magnitude, as a fraction of the body's mean
    /// equatorial radius. Model-dependent (Earth's drifts with epoch).
    pub magnetic_offset_fraction: Measurement,
    /// Offset direction where known — a scalar magnitude alone discards
    /// it. e.g. "northward" for Mercury and Saturn.
    pub offset_direction: Option<String>,
}

/// Component-level descriptors. Eccentricity and arc-confinement belong
/// to specific components (Uranus ε, Neptune Adams), not whole systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RingTrait {
    Uniform,
    Mixed,
    /// At least one component is eccentric (does not circle the center).
    EccentricComponent,
    /// Arcs are density enhancements inside a complete ring — never model
    /// them as a partial ring replacing a whole one (Neptune Adams).
    ArcEnhancements,
    DustDominated,
    OpticallyThick,
    OpticallyThin,
}

/// One ring system. `extent_km` is authoritative — normalized extents are
/// ambiguous for triaxial bodies (Haumea) unless `reference_radius_km` is
/// declared. `component_count` counts named components/complexes with a
/// single convention everywhere (Jupiter: 4; Saturn: 7; Uranus: 13).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RingSystem {
    pub component_count: u16,
    /// (inner, outer) kilometres from body center.
    pub extent_km: (f64, f64),
    /// The reference radius used if a normalized extent is ever derived.
    pub reference_radius_km: Option<f64>,
    pub traits: Vec<RingTrait>,
}

/// Moon inventory. `regular`/`irregular` are orbital classes — they do
/// NOT assert formation history (capture vs in-situ is an interpretation;
/// Mars's moon origins are still debated). `census_epoch` is the source's
/// announcement/effective date — required, exact ISO where it exists.
/// `origin_note` carries per-body provenance hypotheses as text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SatelliteCensus {
    pub confirmed: u16,
    pub regular: u16,
    pub irregular: u16,
    pub census_epoch: String,
    pub origin_note: Option<String>,
}

/// Heliospheric boundary structures — the dynamic shield between solar
/// wind and interstellar medium. They are moving, asymmetric surfaces,
/// not static radial layers: the Voyager shock crossings differ by 10 AU
/// through trajectory and time-dependence, and models infer the shock
/// moved ~8 AU inward under weak solar-wind pressure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeliosphericLayer {
    /// Solar wind goes super-magnetosonic -> subsonic.
    TerminationShock,
    /// Shocked solar-wind plasma between shock and heliopause; extent is
    /// per-trajectory (between that craft's paired crossings), not global.
    Heliosheath,
    /// Contact boundary between solar-wind and interstellar plasmas.
    /// A surface — it has no thickness to tabulate.
    Heliopause,
}

/// A recorded crossing observation: spacecraft, which structure, date,
/// distance, direction. These are observations, not universal layer
/// edges — the distinction is the whole point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundaryCrossing {
    pub spacecraft: String,
    pub structure: HeliosphericLayer,
    pub date: String,
    pub distance_au: f64,
    pub direction: CrossingDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossingDirection {
    Outbound,
    Inbound,
}

/// What a boundary layer's position responds to. Solar-wind *pressure*
/// and solar-cycle activity are distinct drivers — coupling is not just
/// "solar cycle".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryCoupling {
    SolarWindPressure,
    SolarCycleActivity,
}

/// What kind of geometry a body (or boundary) carries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProfileGeometry {
    Planet {
        axial: AxialProfile,
        rings: Vec<RingSystem>,
        moons: SatelliteCensus,
    },
    /// The Sun: axis tilted ~7.25 deg to the ecliptic, global field that
    /// reverses polarity every ~11 years (~22 yr full cycle). Its retinue
    /// is the system itself.
    Star {
        axial_tilt_deg: Measurement,
        /// Interval between polarity reversals (~11 yr), not the full
        /// ~22 yr magnetic cycle.
        field_reversal_interval_years: Measurement,
    },
    /// A heliospheric boundary profile — geometry without a body.
    /// `crossings` are recorded observations; `nominal_au` is optional and
    /// explicitly model-dependent; the heliopause is a surface and carries
    /// no nominal thickness.
    Boundary {
        layer: HeliosphericLayer,
        crossings: Vec<BoundaryCrossing>,
        nominal_au: Option<(f64, f64)>,
        coupling: Vec<BoundaryCoupling>,
    },
}

/// A declared profile: the body, its geometry, and a human-readable note
/// on what makes it distinctive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BodyProfile {
    pub body: Body,
    pub geometry: ProfileGeometry,
    pub notable: String,
}

fn crossing(spacecraft: &str, structure: HeliosphericLayer, date: &str, au: f64) -> BoundaryCrossing {
    BoundaryCrossing {
        spacecraft: spacecraft.into(),
        structure,
        date: date.into(),
        distance_au: au,
        direction: CrossingDirection::Outbound,
    }
}

/// The declared catalog. Sources: NASA/JPL fact sheets, Voyager crossing
/// publications, ring/moon announcements — epochs stamped per entry.
/// Corrected per Athena's review of 3d6c4cc (upper bounds not treated as
/// measurements; Saturn offset restored; Adams arcs are enhancements
/// within a complete ring; crossings are observations, not layer edges).
pub fn solar_system() -> Vec<BodyProfile> {
    vec![
        BodyProfile {
            body: Body::Sun,
            geometry: ProfileGeometry::Star {
                axial_tilt_deg: Measurement::measured(7.25),
                field_reversal_interval_years: Measurement::measured(11.0),
            },
            notable: "field reverses every ~11 yr (full cycle ~22); the system's anchor".into(),
        },
        BodyProfile {
            body: Body::Mercury,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Measurement::measured(0.034),
                    magnetic_tilt_deg: Measurement::UpperBound { limit: 0.8 },
                    magnetic_offset_fraction: Measurement::measured(0.195),
                    offset_direction: Some("northward".into()),
                },
                rings: vec![],
                moons: SatelliteCensus {
                    confirmed: 0,
                    regular: 0,
                    irregular: 0,
                    census_epoch: "2025-01".into(),
                    origin_note: None,
                },
            },
            notable: "weak field offset ~0.2 R northward; nearly untilted".into(),
        },
        BodyProfile {
            body: Body::Venus,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Measurement::measured(177.4), // retrograde
                    magnetic_tilt_deg: Measurement::Absent,      // no intrinsic dipole
                    magnetic_offset_fraction: Measurement::NotApplicable,
                    offset_direction: None,
                },
                rings: vec![],
                moons: SatelliteCensus {
                    confirmed: 0,
                    regular: 0,
                    irregular: 0,
                    census_epoch: "2025-01".into(),
                    origin_note: None,
                },
            },
            notable: "retrograde rotation; induced magnetosphere only — no intrinsic field".into(),
        },
        BodyProfile {
            body: Body::Earth,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Measurement::measured(23.44),
                    magnetic_tilt_deg: Measurement::measured(11.0), // epoch-dependent
                    magnetic_offset_fraction: Measurement::measured(0.088), // modern eccentric-dipole ~0.085-0.091
                    offset_direction: Some("northward".into()),
                },
                rings: vec![],
                moons: SatelliteCensus {
                    confirmed: 1,
                    regular: 1,
                    irregular: 0,
                    census_epoch: "2025-01".into(),
                    origin_note: Some("giant impact".into()),
                },
            },
            notable: "one moon, anomalously large — its torque stabilizes the obliquity".into(),
        },
        BodyProfile {
            body: Body::Mars,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Measurement::measured(25.19),
                    magnetic_tilt_deg: Measurement::Absent, // strong crustal remanence, no global dipole
                    magnetic_offset_fraction: Measurement::NotApplicable,
                    offset_direction: None,
                },
                rings: vec![],
                moons: SatelliteCensus {
                    confirmed: 2,
                    regular: 2,
                    irregular: 0,
                    census_epoch: "2025-01".into(),
                    origin_note: Some("capture vs in-situ debated".into()),
                },
            },
            notable: "Phobos is decaying — it will shear into a ring within ~50 Myr".into(),
        },
        BodyProfile {
            body: Body::Jupiter,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Measurement::measured(3.13),
                    magnetic_tilt_deg: Measurement::measured_model(10.25, "JRM33"),
                    // single-scalar offset is model-limited; modern Jovian
                    // fields are not adequately eccentric-dipole
                    magnetic_offset_fraction: Measurement::measured_model(0.11, "eccentric-dipole (older)"),
                    offset_direction: None,
                },
                rings: vec![RingSystem {
                    component_count: 4, // halo, main, Amalthea gossamer, Thebe gossamer
                    extent_km: (90_000.0, 250_000.0),
                    reference_radius_km: Some(71_492.0),
                    traits: vec![RingTrait::DustDominated, RingTrait::OpticallyThin],
                }],
                moons: SatelliteCensus {
                    confirmed: 95,
                    regular: 8,
                    irregular: 87,
                    census_epoch: "2024-01".into(), // snapshot epoch; NASA listed 115 by 2026-04
                    origin_note: None,
                },
            },
            notable: "largest magnetosphere in the system; faint dust ring with 4 components".into(),
        },
        BodyProfile {
            body: Body::Saturn,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Measurement::measured(26.73),
                    magnetic_tilt_deg: Measurement::UpperBound { limit: 0.007 }, // bound, not measurement
                    // magnetic-equator displacement ~0.0468 R_S northward —
                    // "freakishly aligned" tilt does NOT mean zero offset
                    magnetic_offset_fraction: Measurement::measured(0.0468),
                    offset_direction: Some("northward".into()),
                },
                rings: vec![RingSystem {
                    component_count: 7, // D through E groups; A/B/C bright, D/E/F/G faint
                    extent_km: (66_900.0, 480_000.0), // D inner edge to diffuse E outer
                    reference_radius_km: Some(60_268.0),
                    traits: vec![RingTrait::Mixed], // thick A/B/C + faint D/E/F/G
                }],
                moons: SatelliteCensus {
                    confirmed: 274,
                    regular: 24,
                    irregular: 250,
                    census_epoch: "2025-03-11".into(), // all 128 additions were irregular; NASA listed 293 by 2026-06
                    origin_note: None,
                },
            },
            notable: "the canonical ring system; tilt <0.007 deg but dipole displaced ~0.047 R_S northward".into(),
        },
        BodyProfile {
            body: Body::Uranus,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Measurement::measured(97.77), // rolls on its side
                    magnetic_tilt_deg: Measurement::measured(58.6),
                    magnetic_offset_fraction: Measurement::measured(0.305),
                    offset_direction: None,
                },
                rings: vec![RingSystem {
                    component_count: 13,
                    extent_km: (38_000.0, 98_000.0), // ~1.49-3.84 equatorial radii
                    reference_radius_km: Some(25_559.0),
                    traits: vec![RingTrait::EccentricComponent, RingTrait::OpticallyThin],
                }],
                moons: SatelliteCensus {
                    confirmed: 28,
                    regular: 18,
                    irregular: 10,
                    census_epoch: "2024-02-23".into(), // 28th moon announced this date
                    origin_note: None,
                },
            },
            notable: "most skewed field in the system; rings roll sideways with the planet; epsilon ring is eccentric".into(),
        },
        BodyProfile {
            body: Body::Neptune,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Measurement::measured(28.32),
                    magnetic_tilt_deg: Measurement::measured(46.9),
                    magnetic_offset_fraction: Measurement::measured(0.55), // largest known offset
                    offset_direction: None,
                },
                rings: vec![RingSystem {
                    component_count: 5,
                    extent_km: (42_000.0, 62_930.0), // ~1.70-2.54 radii
                    reference_radius_km: Some(24_764.0),
                    traits: vec![RingTrait::ArcEnhancements, RingTrait::OpticallyThin],
                }],
                moons: SatelliteCensus {
                    confirmed: 16,
                    regular: 7,
                    irregular: 9,
                    census_epoch: "2024-02-23".into(),
                    origin_note: Some("Triton: retrograde — unusually strong capture evidence".into()),
                },
            },
            notable: "Adams is a complete ring with confined bright arcs; Triton is a captured KBO".into(),
        },
        BodyProfile {
            body: Body::Pluto,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Measurement::measured(119.6), // positive-pole convention; ~57 vs orbital plane
                    magnetic_tilt_deg: Measurement::Unknown,
                    magnetic_offset_fraction: Measurement::Unknown,
                    offset_direction: None,
                },
                rings: vec![],
                moons: SatelliteCensus {
                    confirmed: 5,
                    regular: 5,
                    irregular: 0,
                    census_epoch: "2025-01".into(),
                    origin_note: Some("Charon: giant impact hypothesis".into()),
                },
            },
            notable: "Charon barycenter sits outside the surface — a mutually-locked double system".into(),
        },
        BodyProfile {
            body: Body::Haumea,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    // occultation + dynamical work constrain the pole near
                    // RA 285.1, Dec -10.6 (~14 deg obliquity reported)
                    obliquity_deg: Measurement::measured(14.0),
                    magnetic_tilt_deg: Measurement::Unknown,
                    magnetic_offset_fraction: Measurement::Unknown,
                    offset_direction: None,
                },
                rings: vec![RingSystem {
                    component_count: 1, // discovered 2017 via occultation, ~70 km wide
                    extent_km: (2_252.0, 2_322.0),
                    // triaxial body — normalized extent ambiguous; km authoritative
                    reference_radius_km: None,
                    traits: vec![RingTrait::OpticallyThin],
                }],
                moons: SatelliteCensus {
                    confirmed: 2, // Hi'iaka, Namaka
                    regular: 2,
                    irregular: 0,
                    census_epoch: "2025-01".into(),
                    origin_note: Some("impact-family hypothesis".into()),
                },
            },
            notable: "triaxial dwarf with a ring and two moons; ~4-hour rotation; pole constrained".into(),
        },
        // Heliospheric boundary structures — recorded crossings, not
        // universal layer spans. All Voyager crossings were outbound.
        BodyProfile {
            body: Body::Sun,
            geometry: ProfileGeometry::Boundary {
                layer: HeliosphericLayer::TerminationShock,
                crossings: vec![
                    crossing("Voyager 1", HeliosphericLayer::TerminationShock, "2004-12-16", 94.0),
                    crossing("Voyager 2", HeliosphericLayer::TerminationShock, "2007-08-30", 84.0),
                ],
                // nominal span is model-dependent; shock inferred ~8 AU
                // inward under weak solar-wind pressure
                nominal_au: Some((84.0, 94.0)),
                coupling: vec![BoundaryCoupling::SolarWindPressure, BoundaryCoupling::SolarCycleActivity],
            },
            notable: "V1 crossed at 94 AU (2004), V2 at 84 (2007) — asymmetry + time dependence, not a fixed radius".into(),
        },
        BodyProfile {
            body: Body::Sun,
            geometry: ProfileGeometry::Boundary {
                layer: HeliosphericLayer::Heliosheath,
                // extent is per-trajectory: the span between each craft's
                // paired shock and heliopause crossings
                crossings: vec![
                    crossing("Voyager 1", HeliosphericLayer::TerminationShock, "2004-12-16", 94.0),
                    crossing("Voyager 1", HeliosphericLayer::Heliopause, "2012-08-25", 121.7),
                    crossing("Voyager 2", HeliosphericLayer::TerminationShock, "2007-08-30", 84.0),
                    crossing("Voyager 2", HeliosphericLayer::Heliopause, "2018-11-05", 119.0),
                ],
                nominal_au: None, // derived per trajectory, never a global span
                coupling: vec![BoundaryCoupling::SolarWindPressure, BoundaryCoupling::SolarCycleActivity],
            },
            notable: "turbulent shocked plasma; ~35 AU wide on V1's path, ~27 on V2's — trajectory-local".into(),
        },
        BodyProfile {
            body: Body::Sun,
            geometry: ProfileGeometry::Boundary {
                layer: HeliosphericLayer::Heliopause,
                crossings: vec![
                    crossing("Voyager 1", HeliosphericLayer::Heliopause, "2012-08-25", 121.7),
                    crossing("Voyager 2", HeliosphericLayer::Heliopause, "2018-11-05", 119.0),
                ],
                nominal_au: None, // a surface — has no thickness to tabulate
                coupling: vec![BoundaryCoupling::SolarWindPressure, BoundaryCoupling::SolarCycleActivity],
            },
            notable: "the plasma contact boundary — moves with wind pressure; not the edge of all solar influence".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_covers_solar_system() {
        let profiles = solar_system();
        let bodies: Vec<Body> = profiles.iter().map(|p| p.body).collect();
        for b in [
            Body::Sun, Body::Mercury, Body::Venus, Body::Earth, Body::Mars,
            Body::Jupiter, Body::Saturn, Body::Uranus, Body::Neptune,
            Body::Pluto, Body::Haumea,
        ] {
            assert!(bodies.contains(&b), "missing {b:?}");
        }
    }

    #[test]
    fn planet_values_are_physical() {
        for p in solar_system() {
            if let ProfileGeometry::Planet { axial, rings, moons } = &p.geometry {
                if let Some(o) = axial.obliquity_deg.value() {
                    assert!((0.0..=180.0).contains(&o), "{:?} obliquity", p.body);
                }
                if let Some(t) = axial.magnetic_tilt_deg.value() {
                    assert!((0.0..=90.0).contains(&t), "{:?} tilt", p.body);
                }
                if let Some(f) = axial.magnetic_offset_fraction.value() {
                    assert!((0.0..1.0).contains(&f), "{:?} offset", p.body);
                }
                for r in rings {
                    assert!(r.extent_km.0 < r.extent_km.1, "{:?} ring extent", p.body);
                    assert!(r.component_count > 0);
                }
                assert!(moons.regular + moons.irregular <= moons.confirmed, "{:?} census", p.body);
                assert!(moons.census_epoch.len() >= 4, "{:?} epoch", p.body);
            }
        }
    }

    #[test]
    fn epistemic_states_are_distinct() {
        let profiles = solar_system();
        let get = |b: Body| {
            profiles.iter().find_map(|p| {
                if p.body == b {
                    if let ProfileGeometry::Planet { axial, .. } = &p.geometry {
                        return Some(axial);
                    }
                }
                None
            })
        };
        // absent vs unknown vs bounded vs measured are four different claims
        assert_eq!(get(Body::Venus).unwrap().magnetic_tilt_deg, Measurement::Absent);
        assert_eq!(get(Body::Mars).unwrap().magnetic_tilt_deg, Measurement::Absent);
        assert_eq!(get(Body::Pluto).unwrap().magnetic_tilt_deg, Measurement::Unknown);
        assert_eq!(get(Body::Haumea).unwrap().magnetic_tilt_deg, Measurement::Unknown);
        assert!(matches!(
            get(Body::Saturn).unwrap().magnetic_tilt_deg,
            Measurement::UpperBound { .. }
        ));
        assert!(matches!(
            get(Body::Mercury).unwrap().magnetic_tilt_deg,
            Measurement::UpperBound { .. }
        ));
        assert!(matches!(
            get(Body::Earth).unwrap().magnetic_tilt_deg,
            Measurement::Measured { .. }
        ));
        // no intrinsic field => offset is not applicable, not unknown
        assert_eq!(get(Body::Venus).unwrap().magnetic_offset_fraction, Measurement::NotApplicable);
    }

    #[test]
    fn saturn_offset_is_not_zero() {
        // regression: "aligned axis" never meant "centered dipole"
        let profiles = solar_system();
        let saturn = profiles.iter().find(|p| p.body == Body::Saturn).unwrap();
        if let ProfileGeometry::Planet { axial, .. } = &saturn.geometry {
            let f = axial.magnetic_offset_fraction.value().unwrap();
            assert!(f > 0.04, "saturn offset must reflect ~0.047 R_S, got {f}");
        }
    }

    #[test]
    fn ring_traits_are_component_level() {
        let profiles = solar_system();
        let uranus = profiles.iter().find(|p| p.body == Body::Uranus).unwrap();
        if let ProfileGeometry::Planet { rings, .. } = &uranus.geometry {
            assert!(rings[0].traits.contains(&RingTrait::EccentricComponent));
            // extent must exceed 2.0 R_U in km (38k-98k km / 25.6k km radius)
            assert!(rings[0].extent_km.1 / rings[0].reference_radius_km.unwrap() > 2.0);
        }
        let neptune = profiles.iter().find(|p| p.body == Body::Neptune).unwrap();
        if let ProfileGeometry::Planet { rings, .. } = &neptune.geometry {
            // arcs enhance a complete ring — never partial-replacement
            assert!(rings[0].traits.contains(&RingTrait::ArcEnhancements));
        }
    }

    #[test]
    fn census_epochs_are_real_dates() {
        let profiles = solar_system();
        let saturn = profiles.iter().find(|p| p.body == Body::Saturn).unwrap();
        if let ProfileGeometry::Planet { moons, .. } = &saturn.geometry {
            // announcement date, not an arbitrary month
            assert_eq!(moons.census_epoch, "2025-03-11");
        }
        let uranus = profiles.iter().find(|p| p.body == Body::Uranus).unwrap();
        if let ProfileGeometry::Planet { moons, .. } = &uranus.geometry {
            assert_eq!(moons.census_epoch, "2024-02-23");
        }
    }

    #[test]
    fn boundary_crossings_are_observations() {
        let profiles = solar_system();
        let boundaries: Vec<&BodyProfile> = profiles
            .iter()
            .filter(|p| matches!(p.geometry, ProfileGeometry::Boundary { .. }))
            .collect();
        assert_eq!(boundaries.len(), 3);
        for b in &boundaries {
            if let ProfileGeometry::Boundary { layer, crossings, nominal_au, .. } = &b.geometry {
                assert!(!crossings.is_empty(), "{layer:?} has no recorded crossings");
                for c in crossings {
                    assert!(c.distance_au > 0.0);
                    assert!(c.date.len() >= 8, "crossing date should be ISO");
                }
                // heliopause is a surface, not a layer — no nominal span
                if *layer == HeliosphericLayer::Heliopause {
                    assert!(nominal_au.is_none());
                }
            }
        }
        // the 10 AU Voyager asymmetry must be visible in the record
        let ts = boundaries
            .iter()
            .find(|p| matches!(&p.geometry, ProfileGeometry::Boundary { layer: HeliosphericLayer::TerminationShock, .. }))
            .unwrap();
        if let ProfileGeometry::Boundary { crossings, .. } = &ts.geometry {
            let v1 = crossings.iter().find(|c| c.spacecraft == "Voyager 1").unwrap();
            let v2 = crossings.iter().find(|c| c.spacecraft == "Voyager 2").unwrap();
            assert!((v1.distance_au - v2.distance_au).abs() >= 9.0);
        }
    }

    #[test]
    fn profiles_serialize() {
        for p in solar_system() {
            let s = serde_json::to_string(&p).unwrap();
            let back: BodyProfile = serde_json::from_str(&s).unwrap();
            assert_eq!(back, p);
        }
    }
}
