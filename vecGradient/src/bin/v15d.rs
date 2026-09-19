//! v15d — stdio bridge to the canonical Vector15D type.
//!
//! Lets non-Rust callers (pi extensions, shell scripts, agent harnesses)
//! produce codec-valid state instead of hand-rolling field names/values.
//! Contract: stdout is always JSON. Errors are JSON on stdout + exit 1.
//!
//!   v15d fields                 → ordered field manifest (names, kinds, variants)
//!   v15d default                → Vector15D::default() as canonical JSON
//!   v15d normalize  < stdin     → partial JSON merged onto default, validated, canonical out
//!   v15d validate   < stdin     → parse + validate, echo canonical JSON
//!   v15d describe   < stdin     → normalize then add hue/prestress/glyph view
//!   v15d observed               → observed_n1()/observed_n2() baselines

use serde_json::{json, Value};
use std::io::Read;
use vecGradient::{observed_n1, observed_n2, Vector15D};

fn fail(msg: impl std::fmt::Display) -> ! {
    println!("{}", json!({ "ok": false, "error": msg.to_string() }));
    std::process::exit(1);
}

fn read_stdin() -> String {
    let mut buf = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
        fail(format!("stdin read failed: {e}"));
    }
    buf
}

/// Canonical key set comes from the serialized type itself — no duplicated
/// field list that can drift from lib.rs.
fn canonical_keys() -> Vec<String> {
    match serde_json::to_value(Vector15D::default()) {
        Ok(Value::Object(map)) => map.keys().cloned().collect(),
        _ => fail("could not serialize Vector15D::default()"),
    }
}

/// Merge a partial JSON object onto Vector15D::default(), then deserialize
/// through the real type and run validate(). Unknown keys are rejected so
/// typos can't silently pass through the bridge.
fn normalize(input: &str) -> Vector15D {
    let value: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => fail(format!("invalid JSON: {e}")),
    };
    let obj = match value {
        Value::Object(o) => o,
        _ => fail("expected a JSON object of vector fields"),
    };
    let keys = canonical_keys();
    for k in obj.keys() {
        if !keys.contains(k) {
            fail(format!(
                "unknown field '{k}' — canonical fields: {}",
                keys.join(", ")
            ));
        }
    }
    let mut base = match serde_json::to_value(Vector15D::default()) {
        Ok(Value::Object(m)) => m,
        _ => unreachable!(),
    };
    for (k, v) in obj {
        base.insert(k, v);
    }
    let vec: Vector15D = match serde_json::from_value(Value::Object(base)) {
        Ok(v) => v,
        Err(e) => fail(format!("codec rejected fields: {e}")),
    };
    if let Err(e) = vec.validate() {
        fail(format!("{e}"));
    }
    vec
}

fn main() {
    let cmd = std::env::args().nth(1).unwrap_or_default();
    match cmd.as_str() {
        "fields" => {
            // Manifest is descriptive output; validation always derives from
            // the serialized type above.
            println!(
                "{}",
                json!({
                    "ok": true,
                    "codec": "Vector15D",
                    "state_space": "R^11 × {Linked,Broken,Gradient} × {Static,Spinning,Oscillating} + poles",
                    "fields": [
                        {"n": 1,  "name": "amplitude",      "kind": "f64"},
                        {"n": 2,  "name": "frequency",      "kind": "f64"},
                        {"n": 3,  "name": "phase",          "kind": "f64"},
                        {"n": 4,  "name": "coherence",      "kind": "f64"},
                        {"n": 5,  "name": "entropy",        "kind": "f64"},
                        {"n": 6,  "name": "composition",    "kind": "f64", "note": "truth meter"},
                        {"n": 7,  "name": "resonance",      "kind": "f64"},
                        {"n": 8,  "name": "ozone_buffer",   "kind": "f64", "note": "lightness"},
                        {"n": 9,  "name": "domain_wall",    "kind": "enum", "variants": ["Linked","Broken","Gradient"]},
                        {"n": 10, "name": "su2_polarity",   "kind": "f64", "note": "hue degrees [0,360)"},
                        {"n": 11, "name": "torsion",        "kind": "f64", "note": "skew / temporal lean"},
                        {"n": 12, "name": "gauge_coupling", "kind": "enum", "variants": ["Static","Spinning","Oscillating"]},
                        {"n": 13, "name": "closure",        "kind": "f64"},
                        {"n": 14, "name": "magnetic_north", "kind": "f64", "default": 0.0, "note": "universal pole"},
                        {"n": 15, "name": "magnetic_south", "kind": "f64", "default": 0.0, "note": "universal pole"},
                    ]
                })
            );
        }
        "default" => {
            println!(
                "{}",
                json!({ "ok": true, "vector": Vector15D::default() })
            );
        }
        "normalize" | "validate" => {
            let v = normalize(&read_stdin());
            println!("{}", json!({ "ok": true, "vector": v }));
        }
        "describe" => {
            let v = normalize(&read_stdin());
            match v.to_glyph_props_checked() {
                Ok(glyph) => println!(
                    "{}",
                    json!({
                        "ok": true,
                        "vector": v,
                        "hue": v.hue(),
                        "polar_prestress": v.polar_prestress(),
                        "glyph": glyph,
                    })
                ),
                Err(e) => fail(format!("{e}")),
            }
        }
        "observed" => {
            println!(
                "{}",
                json!({ "ok": true, "n1": observed_n1(), "n2": observed_n2() })
            );
        }
        "--help" | "-h" | "help" | "" => {
            println!(
                "{}",
                json!({
                    "ok": true,
                    "usage": "v15d <fields|default|normalize|validate|describe|observed>  (vector JSON on stdin for normalize/validate/describe)"
                })
            );
        }
        other => fail(format!("unknown command '{other}' — try 'v15d help'")),
    }
}
