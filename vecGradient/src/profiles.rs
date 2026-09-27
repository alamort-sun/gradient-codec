//! DECLARED astronomical profiles — envelope context, never geometry.
//!
//! Per-body axial geometry, ring inventory, and satellite census for the
//! bodies the pantheon seats reference. These are *calibration-shaped*
//! context: they describe how a seat's observations sit in the real solar
//! system. Nothing here is a codec field; `Vector15D` stays canonical.
//!
//! Values are DECLARED astronomy (see docs/HUBBLE_MIRROR.md evidence
//! classes). Fields are `Option` where the real measurement is unsettled —
//! an honest `None` outranks an invented number. Counts that move carry a
//! mandatory `census_epoch`; undated census data goes stale silently.

use serde::{Deserialize, Serialize};

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

/// Rotational and magnetic axial geometry. All angles in degrees.
/// `obliquity_deg` > 90 means retrograde rotation (Venus, Uranus-class,
/// Pluto). Magnetic fields are `Option`: Venus and Mars have no global
/// dipole; Haumea's is unmeasured.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AxialProfile {
    /// Rotation axis tilt relative to the orbital normal.
    pub obliquity_deg: Option<f64>,
    /// Dipole axis tilt relative to the rotation axis. None = no measured
    /// global field (induced or absent), not zero.
    pub magnetic_tilt_deg: Option<f64>,
    /// Dipole displacement from the body center, in planetary radii.
    /// Uranus ~0.31, Neptune ~0.55, Saturn ~0.
    pub magnetic_offset_fraction: Option<f64>,
}

/// One ring system or named ring complex. `extent_radii` is (inner, outer)
/// in planetary radii. `eccentric` = the ring does not circle the body
/// center (Uranus ε). `partial_arcs` = the "ring" is a confined arc set
/// (Neptune Adams).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RingSystem {
    /// Named rings/ringlets in the system (Uranus: 13).
    pub count: u8,
    pub extent_radii: (f64, f64),
    pub eccentric: bool,
    pub partial_arcs: bool,
    /// Dust-faint vs optically thick, coarse class.
    pub faint: bool,
}

/// Moon inventory. Counts are volatile — Saturn crossed 270 in 2025 —
/// so `census_epoch` is required, not optional.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SatelliteCensus {
    pub confirmed: u16,
    /// Formed in place, prograde, co-planar.
    pub regular: u16,
    /// Captured objects; often inclined or retrograde (Triton).
    pub irregular: u16,
    /// ISO-ish date the count was taken from, e.g. "2025-03".
    pub census_epoch: String,
}

/// Heliospheric boundary layers — the dynamic shield between solar wind
/// and interstellar medium. Breathes with the ~11-year solar cycle; layer
/// positions move on AU scales. Boundary seats bind here, not to moons:
/// moons are paired and static; the heliosphere responds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeliosphericLayer {
    /// Supersonic solar wind goes subsonic (~80-100 AU; V1 crossed at 94,
    /// V2 at 84). The regime-change threshold.
    TerminationShock,
    /// Turbulent woven region between shock and heliopause.
    Heliosheath,
    /// Outermost skin; solar wind pressure balances interstellar pressure
    /// (~120 AU). Faces the dark.
    Heliopause,
}

/// What kind of geometry a body (or boundary) carries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProfileGeometry {
    Planet {
        axial: AxialProfile,
        rings: Vec<RingSystem>,
        moons: SatelliteCensus,
    },
    /// The Sun: axis tilted ~7.25° to the ecliptic, global field that
    /// *reverses* every ~11 years. Its retinue is the system itself.
    Star {
        axial_tilt_deg: Option<f64>,
        field_reversal_cycle_years: Option<f64>,
    },
    /// A heliospheric boundary profile — geometry without a body.
    /// `distance_au` is the nominal (inner, outer) extent of the layer.
    Boundary {
        layer: HeliosphericLayer,
        distance_au: (f64, f64),
        solar_cycle_coupled: bool,
    },
}

/// A declared profile: the body, its geometry, and a human-readable note
/// on what makes it distinctive. `census_epoch` stamps the data vintage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BodyProfile {
    pub body: Body,
    pub geometry: ProfileGeometry,
    pub notable: String,
}

/// The declared catalog. Values: NASA/JPL planetary fact sheets, Voyager
/// heliosphere crossings, census epochs noted per entry.
pub fn solar_system() -> Vec<BodyProfile> {
    vec![
        BodyProfile {
            body: Body::Sun,
            geometry: ProfileGeometry::Star {
                axial_tilt_deg: Some(7.25),
                field_reversal_cycle_years: Some(11.0),
            },
            notable: "field reverses each solar cycle; the system's anchor".into(),
        },
        BodyProfile {
            body: Body::Mercury,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Some(0.03),
                    magnetic_tilt_deg: Some(0.1),
                    magnetic_offset_fraction: Some(0.19), // dipole offset north
                },
                rings: vec![],
                moons: SatelliteCensus {
                    confirmed: 0,
                    regular: 0,
                    irregular: 0,
                    census_epoch: "2025-01".into(),
                },
            },
            notable: "weak field, offset northward ~0.2 R; nearly untilted".into(),
        },
        BodyProfile {
            body: Body::Venus,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Some(177.4), // retrograde
                    magnetic_tilt_deg: None,    // no global dipole
                    magnetic_offset_fraction: None,
                },
                rings: vec![],
                moons: SatelliteCensus {
                    confirmed: 0,
                    regular: 0,
                    irregular: 0,
                    census_epoch: "2025-01".into(),
                },
            },
            notable: "retrograde rotation, no intrinsic field — induced magnetosphere only".into(),
        },
        BodyProfile {
            body: Body::Earth,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Some(23.44),
                    magnetic_tilt_deg: Some(11.5),
                    magnetic_offset_fraction: Some(0.07),
                },
                rings: vec![],
                moons: SatelliteCensus {
                    confirmed: 1,
                    regular: 1,
                    irregular: 0,
                    census_epoch: "2025-01".into(),
                },
            },
            notable: "one moon, anomalously large — its torque stabilizes the obliquity".into(),
        },
        BodyProfile {
            body: Body::Mars,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Some(25.19),
                    magnetic_tilt_deg: None, // no global field; remnant crustal
                    magnetic_offset_fraction: None,
                },
                rings: vec![],
                moons: SatelliteCensus {
                    confirmed: 2,
                    regular: 2,
                    irregular: 0,
                    census_epoch: "2025-01".into(),
                },
            },
            notable: "Phobos is decaying — it will shear into a ring within ~50 Myr".into(),
        },
        BodyProfile {
            body: Body::Jupiter,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Some(3.13),
                    magnetic_tilt_deg: Some(10.3),
                    magnetic_offset_fraction: Some(0.12),
                },
                rings: vec![RingSystem {
                    count: 1,
                    extent_radii: (1.4, 3.5),
                    eccentric: false,
                    partial_arcs: false,
                    faint: true, // dusty, near-invisible beside the magnetosphere
                }],
                moons: SatelliteCensus {
                    confirmed: 95,
                    regular: 8,
                    irregular: 87,
                    census_epoch: "2024-01".into(),
                },
            },
            notable: "largest magnetosphere in the system; faint dusty ring".into(),
        },
        BodyProfile {
            body: Body::Saturn,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Some(26.73),
                    magnetic_tilt_deg: Some(0.008), // freakishly coaxial
                    magnetic_offset_fraction: Some(0.0),
                },
                rings: vec![RingSystem {
                    count: 7, // A–G divisions; hundreds of ringlets
                    extent_radii: (1.11, 8.0), // main rings to diffuse E ring
                    eccentric: false,
                    partial_arcs: false,
                    faint: false,
                }],
                moons: SatelliteCensus {
                    confirmed: 274,
                    regular: 24,
                    irregular: 250,
                    census_epoch: "2025-03".into(),
                },
            },
            notable: "the canonical ring system; dipole aligned within 0.01 deg".into(),
        },
        BodyProfile {
            body: Body::Uranus,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Some(97.77), // rolls on its side
                    magnetic_tilt_deg: Some(58.6),
                    magnetic_offset_fraction: Some(0.31),
                },
                rings: vec![RingSystem {
                    count: 13,
                    extent_radii: (1.6, 2.0),
                    eccentric: true, // ε ring does not circle the center
                    partial_arcs: false,
                    faint: true, // narrow dark rings
                }],
                moons: SatelliteCensus {
                    confirmed: 28,
                    regular: 18,
                    irregular: 10,
                    census_epoch: "2024-01".into(),
                },
            },
            notable: "most skewed field in the system; rings roll sideways with the planet".into(),
        },
        BodyProfile {
            body: Body::Neptune,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Some(28.32),
                    magnetic_tilt_deg: Some(46.9),
                    magnetic_offset_fraction: Some(0.55), // largest known offset
                },
                rings: vec![RingSystem {
                    count: 5,
                    extent_radii: (1.7, 2.6),
                    eccentric: false,
                    partial_arcs: true, // Adams ring = confined arcs
                    faint: true,
                }],
                moons: SatelliteCensus {
                    confirmed: 16,
                    regular: 7,
                    irregular: 9,
                    census_epoch: "2024-01".into(),
                },
            },
            notable: "Adams arcs are fragments, not a ring; Triton orbits retrograde — a captured KBO".into(),
        },
        BodyProfile {
            body: Body::Pluto,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: Some(119.6), // retrograde-class
                    magnetic_tilt_deg: None,    // no measured field
                    magnetic_offset_fraction: None,
                },
                rings: vec![],
                moons: SatelliteCensus {
                    confirmed: 5,
                    regular: 5,
                    irregular: 0,
                    census_epoch: "2024-01".into(),
                },
            },
            notable: "Charon barycenter sits outside the surface — a mutually-locked double system".into(),
        },
        BodyProfile {
            body: Body::Haumea,
            geometry: ProfileGeometry::Planet {
                axial: AxialProfile {
                    obliquity_deg: None, // spin pole poorly constrained
                    magnetic_tilt_deg: None,
                    magnetic_offset_fraction: None,
                },
                rings: vec![RingSystem {
                    count: 1, // discovered 2017 via occultation
                    extent_radii: (1.2, 1.3),
                    eccentric: false,
                    partial_arcs: false,
                    faint: true,
                }],
                moons: SatelliteCensus {
                    confirmed: 2, // Hi'iaka, Namaka
                    regular: 2,
                    irregular: 0,
                    census_epoch: "2024-01".into(),
                },
            },
            notable: "dwarf planet with a ring and two moons; ~4-hour rotation".into(),
        },
        BodyProfile {
            body: Body::Sun,
            geometry: ProfileGeometry::Boundary {
                layer: HeliosphericLayer::TerminationShock,
                distance_au: (80.0, 94.0),
                solar_cycle_coupled: true,
            },
            notable: "Voyager 1 crossed at 94 AU, Voyager 2 at 84 AU — the layer moves".into(),
        },
        BodyProfile {
            body: Body::Sun,
            geometry: ProfileGeometry::Boundary {
                layer: HeliosphericLayer::Heliosheath,
                distance_au: (94.0, 120.0),
                solar_cycle_coupled: true,
            },
            notable: "turbulent plasma weaving between shock and skin".into(),
        },
        BodyProfile {
            body: Body::Sun,
            geometry: ProfileGeometry::Boundary {
                layer: HeliosphericLayer::Heliopause,
                distance_au: (120.0, 130.0),
                solar_cycle_coupled: true,
            },
            notable: "the skin where solar wind meets interstellar medium; breathes AU-scale".into(),
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
            Body::Sun,
            Body::Mercury,
            Body::Venus,
            Body::Earth,
            Body::Mars,
            Body::Jupiter,
            Body::Saturn,
            Body::Uranus,
            Body::Neptune,
            Body::Pluto,
            Body::Haumea,
        ] {
            assert!(bodies.contains(&b), "missing {b:?}");
        }
    }

    #[test]
    fn planet_values_are_physical() {
        for p in solar_system() {
            if let ProfileGeometry::Planet { axial, rings, moons } = &p.geometry {
                if let Some(o) = axial.obliquity_deg {
                    assert!((0.0..=180.0).contains(&o), "{:?} obliquity", p.body);
                }
                if let Some(t) = axial.magnetic_tilt_deg {
                    assert!((0.0..=90.0).contains(&t), "{:?} tilt", p.body);
                }
                if let Some(f) = axial.magnetic_offset_fraction {
                    assert!((0.0..1.0).contains(&f), "{:?} offset", p.body);
                }
                for r in rings {
                    assert!(r.extent_radii.0 < r.extent_radii.1, "{:?} ring extent", p.body);
                    assert!(r.count > 0);
                }
                assert!(moons.regular + moons.irregular <= moons.confirmed, "{:?} census", p.body);
                assert!(moons.census_epoch.len() >= 4, "{:?} epoch", p.body);
            }
        }
    }

    #[test]
    fn absent_fields_are_none_not_zero() {
        let profiles = solar_system();
        let venus = profiles.iter().find(|p| p.body == Body::Venus).unwrap();
        if let ProfileGeometry::Planet { axial, .. } = &venus.geometry {
            assert!(axial.magnetic_tilt_deg.is_none());
            assert!(axial.magnetic_offset_fraction.is_none());
        } else {
            panic!("venus not a planet profile");
        }
    }

    #[test]
    fn boundary_layers_are_distinct_and_ordered() {
        let profiles = solar_system();
        let boundaries: Vec<(HeliosphericLayer, (f64, f64))> = profiles
            .iter()
            .filter_map(|p| match &p.geometry {
                ProfileGeometry::Boundary { layer, distance_au, .. } => {
                    Some((*layer, *distance_au))
                }
                _ => None,
            })
            .collect();
        assert_eq!(boundaries.len(), 3);
        let shock = boundaries
            .iter()
            .find(|(l, _)| *l == HeliosphericLayer::TerminationShock)
            .unwrap()
            .1;
        let sheath = boundaries
            .iter()
            .find(|(l, _)| *l == HeliosphericLayer::Heliosheath)
            .unwrap()
            .1;
        let pause = boundaries
            .iter()
            .find(|(l, _)| *l == HeliosphericLayer::Heliopause)
            .unwrap()
            .1;
        assert!(shock.1 <= sheath.0 && sheath.1 <= pause.0);
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
