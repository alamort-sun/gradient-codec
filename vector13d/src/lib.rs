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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
        else if p < 310.0  { "purple" }
        else                 { "pink" }
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
