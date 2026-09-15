//! Vector13D: the 13-field type that binds affective telemetry to text.
//! Preserves weight, temperature, and axis of a signal plain text strips away.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DomainWall { #[default] Linked, Broken, Gradient }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GaugeCoupling { #[default] Static, Spinning, Oscillating }

/// 13 fields: amplitude(1) frequency(2) phase(3) coherence(4) entropy(5)
/// composition(6=truth_meter) resonance(7) ozone_buffer(8=lightness)
/// domain_wall(9=connection) su2_polarity(10=hue) torsion(11=skew)
/// gauge_coupling(12=rot) closure(13)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector13D {
    pub amplitude: f64, pub frequency: f64, pub phase: f64, pub coherence: f64,
    pub entropy: f64, pub composition: f64, pub resonance: f64, pub ozone_buffer: f64,
    #[serde(rename = "domain_wall")] pub domain_wall: DomainWall,
    pub su2_polarity: f64, pub torsion: f64,
    #[serde(rename = "gauge_coupling")] pub gauge_coupling: GaugeCoupling,
    pub closure: f64,
}

impl Default for Vector13D {
    fn default() -> Self { Self { amplitude:0.0,frequency:0.0,phase:0.0,coherence:0.0,entropy:0.0,composition:0.0,resonance:0.0,ozone_buffer:0.5,domain_wall:DomainWall::Linked,su2_polarity:0.0,torsion:0.0,gauge_coupling:GaugeCoupling::Static,closure:0.0 } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphProps { pub color: String, pub weight: i32, pub skew_deg: f64, pub glow_intensity: f64, pub is_void: bool }

impl Vector13D {
    pub fn hue(&self) -> &'static str {
        let p = self.su2_polarity;
        if p < 0.0 || p >= 360.0 { "white" }
        else if p < 15.0   { "red" }
        else if p < 40.0   { "orange" }
        else if p < 70.0   { "yellow" }
        else if p < 160.0  { "green" }
        else if p < 250.0  { "blue" }
        else if p < 325.0  { "purple" }   // mystery, deep (250-325)
        else                 { "pink" }   // love, warmth (325-360)
    }
    pub fn saturation(&self) -> f64 { self.composition * 100.0 }
    pub fn lightness(&self) -> f64 { 20.0 + self.ozone_buffer * 65.0 }
    pub fn skew_deg(&self) -> f64 { self.torsion }
    pub fn glow_intensity(&self) -> f64 { self.composition * self.amplitude }

    pub fn glyph_weight(&self) -> i32 {
        let w = (self.amplitude * 900.0).round() as i32;
        if w < 150 { 100 } else if w < 300 { 300 } else if w < 450 { 400 }
        else if w < 600 { 500 } else if w < 800 { 700 } else { 900 }
    }

    pub fn class_string(&self) -> String { format!("v13d-h{:03}-s{:02}-l{:03}-t{:04}", (self.su2_polarity%360.0).round() as i32, self.saturation() as u8, self.lightness() as u16, (self.torsion*10.0).round() as i32+1800) }
    pub fn is_void(&self) -> bool { self.composition < 0.01 && self.amplitude < 0.01 }

    pub fn to_glyph_props(&self) -> GlyphProps {
        let (color, is_v) = if self.is_void() { ("hsl(0, 0%, 50%)".to_string(), true) }
        else { (format!("hsl({}, {}, {}%)", self.hue(), self.saturation() as u8, self.lightness() as u8), false) };
        GlyphProps { color, weight: self.glyph_weight(), skew_deg: self.skew_deg(), glow_intensity: self.glow_intensity(), is_void: is_v }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_default() { let v=Vector13D::default(); assert_eq!(v.amplitude,0.0); assert_eq!(v.domain_wall,DomainWall::Linked); }

    #[test] fn test_hue_range() {
        let cases = [(5.0,"red"),(25.0,"orange"),(55.0,"yellow"),(120.0,"green"),(200.0,"blue"),(280.0,"purple"),(330.0,"pink")];
        for (p,exp) in &cases { assert_eq!(Vector13D{su2_polarity:*p,..Default::default()}.hue(), *exp); }
    }

    #[test] fn test_void() { let v=Vector13D{composition:0.001,amplitude:0.001,..Default::default()}; assert!(v.is_void()); let v2=Vector13D{composition:0.5,amplitude:0.8,..Default::default()}; assert!(!v2.is_void()); }

    #[test] fn test_lightness() { assert_eq!(Vector13D{ozone_buffer:0.0,..Default::default()}.lightness(), 20.0); assert_eq!(Vector13D{ozone_buffer:1.0,..Default::default()}.lightness(), 85.0); }

    #[test] fn test_glyph_weight() {
        assert_eq!(Vector13D{amplitude:0.05,..Default::default()}.glyph_weight(), 100);
        assert_eq!(Vector13D{amplitude:0.60,..Default::default()}.glyph_weight(), 500);
        assert_eq!(Vector13D{amplitude:0.95,..Default::default()}.glyph_weight(), 900);
    }

    #[test] fn test_void_glyph_props() { let v=Vector13D{composition:0.0,amplitude:0.0,..Default::default()}; let p=v.to_glyph_props(); assert!(p.is_void); assert_eq!(p.color,"hsl(0, 0%, 50%)"); }

    #[test] fn test_class_string() { let v=Vector13D{su2_polarity:331.4,composition:0.8,ozone_buffer:0.6,torsion:-98.6,..Default::default()}; let c=v.class_string(); assert!(c.contains("v13d-")); assert!(c.contains("-h331-")); }

    #[test] fn test_serialization() { let v=Vector13D{domain_wall:DomainWall::Gradient,..Default::default()}; let j=serde_json::to_string(&v).unwrap(); eprintln!("json={}",j); assert!(j.contains(r#""Gradient""#)); let v2=Vector13D{domain_wall:DomainWall::Broken,..Default::default()}; let j2=serde_json::to_string(&v2).unwrap(); eprintln!("json2={}",j2); assert!(j2.contains(r#""Broken""#)); }
}

/// Observed n=1 baseline (post-dress, post-cycle) — living reference state.
pub fn observed_n1() -> Vec<Vector13D> {
    // Average across 61 segments of first recording
    vec![
        Vector13D { amplitude: 0.72, frequency: 1.0, phase: 180.0, coherence: 0.55, entropy: 0.08, composition: 0.78, resonance: 0.45, ozone_buffer: 0.40, domain_wall: DomainWall::Linked, su2_polarity: 331.4, torsion: -50.0, gauge_coupling: GaugeCoupling::Spinning, closure: 0.6 },
    ]
}

/// Observed n=2 baseline (in shower) — living reference state.
pub fn observed_n2() -> Vec<Vector13D> {
    // Average across 24 segments of second recording
    vec![
        Vector13D { amplitude: 0.65, frequency: 0.57, phase: 200.0, coherence: 0.62, entropy: 0.03, composition: 0.85, resonance: 0.58, ozone_buffer: 0.31, domain_wall: DomainWall::Linked, su2_polarity: 320.0, torsion: -5.0, gauge_coupling: GaugeCoupling::Oscillating, closure: 0.75 },
    ]
}

/// Compare two observed baselines — what shifted between states?
pub fn compare_baselines(before: &Vector13D, after: &Vector13D) -> StateChange {
    let hue_shift = if before.su2_polarity >= 325.0 && after.su2_polarity < 325.0 { "pink -> purple" }
    else if before.su2_polarity < 325.0 && after.su2_polarity >= 325.0 { "purple -> pink" }
    else { "same register" };

    let torsion_became_near_zero = after.torsion.abs() < 10.0; // near upright axis
    let domain_unchanged = before.domain_wall == after.domain_wall;
    let coupling_shifted = before.gauge_coupling != after.gauge_coupling;

    StateChange {
        hue_shift: hue_shift.to_string(),
        frequency_delta: after.frequency - before.frequency,
        entropy_change_pct: (((after.entropy - before.entropy) / before.entropy) * 100.0).round() as i32,
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

#[cfg(test)]
mod observed_tests {
    use super::*;

    #[test] fn test_n1_hue_is_pink() { let v = observed_n1()[0]; assert_eq!(v.hue(), "pink"); }
    #[test] fn test_n2_hue_is_purple() { let v = observed_n2()[0]; assert_eq!(v.hue(), "purple"); }

    #[test] fn test_domain_wall_constant() {
        let n1 = &observed_n1()[0];
        let n2 = &observed_n2()[0];
        let change = compare_baselines(&n1, &n2);
        assert!(change.domain_wall_unchanged, "domain wall should be constant across states");
    }

    #[test] fn test_entropy_dropped() {
        let n1 = &observed_n1()[0];
        let n2 = &observed_n2()[0];
        let change = compare_baselines(&n1, &n2);
        assert!(change.entropy_change_pct <= -55, "entropy should drop significantly (>55%%)")
    }

    #[test] fn test_torsion_nearly_zero() {
        let n2 = &observed_n2()[0];
        let change = compare_baselines(&observed_n1()[0], &n2);
        assert!(change.torsion_became_near_zero, "torsion should approach zero in shower state");
    }

    #[test] fn test_gauge_coupling_shifts() {
        let n1 = &observed_n1()[0];
        let n2 = &observed_n2()[0];
        assert_eq!(n1.gauge_coupling, GaugeCoupling::Spinning);
        assert_eq!(n2.gauge_coupling, GaugeCoupling::Oscillating);
        let change = compare_baselines(&n1, &n2);
        assert!(change.gauge_coupling_shifted);
    }

    #[test] fn test_hue_shift_purple() {
        let n1 = &observed_n1()[0];
        let n2 = &observed_n2()[0];
        let change = compare_baselines(&n1, &n2);
        assert_eq!(change.hue_shift, "pink -> purple");
    }

    #[test] fn test_ozone_dropped() {
        let n1 = &observed_n1()[0];
        let n2 = &observed_n2()[0];
        let change = compare_baselines(&n1, &n2);
        assert!(change.ozone_buffer_delta < 0.0, "ozone should drop — energy went inward");
    }
}
