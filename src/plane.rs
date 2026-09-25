//! Plane split (P0.3) — orchestration vs geometry.
//!
//! This package's bin (`gradient-codec`) is the **budgeted LLM router**.
//! Canonical Vector15D geometry lives in workspace member `vecGradient`
//! (CLI bridge: `v15d`). Downstream cores pin `vecGradient`, not this
//! router package.
//!
//! Coupling the router to `vecGradient` with a token import would be
//! costume: the bin would still be an LLM router pretending to be the
//! geometry authority. Fail-when tests below seal the split.

#![allow(dead_code)]

/// Marker: router plane does not own geometry law.
pub const ROUTER_PLANE: &str = "orchestration";

/// Marker: geometry plane package name (workspace member).
pub const GEOMETRY_PLANE_PACKAGE: &str = "vecGradient";

/// Marker: geometry CLI bin name under vecGradient.
pub const GEOMETRY_CLI_BIN: &str = "v15d";

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn manifest_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    /// `[dependencies]` / `[dev-dependencies]` must not list vecGradient.
    /// A path dep here would be costume coupling of the router plane.
    fn section_mentions_vecgradient(toml: &str, header: &str) -> bool {
        let Some(rest) = toml.split(header).nth(1) else {
            return false;
        };
        let section = rest.split("\n[").next().unwrap_or(rest);
        section.lines().any(|line| {
            let t = line.trim();
            if t.is_empty() || t.starts_with('#') {
                return false;
            }
            t.starts_with("vecGradient")
                || t.contains("vecGradient =")
                || t.contains("/vecGradient")
        })
    }

    #[test]
    fn router_cargo_toml_does_not_depend_on_vecgradient() {
        let cargo = fs::read_to_string(manifest_dir().join("Cargo.toml"))
            .expect("read root Cargo.toml");
        assert!(
            !section_mentions_vecgradient(&cargo, "[dependencies]"),
            "gradient-codec [dependencies] must not list vecGradient — geometry is the vecGradient member, not a token import on the router"
        );
        assert!(
            !section_mentions_vecgradient(&cargo, "[dev-dependencies]"),
            "gradient-codec [dev-dependencies] must not list vecGradient — fail-when tests prove absence, they must not soft-couple"
        );
    }

    #[test]
    fn router_sources_do_not_import_vecgradient() {
        let src = manifest_dir().join("src");
        let mut offenders = Vec::new();
        for entry in fs::read_dir(&src).expect("read src/") {
            let entry = entry.expect("dirent");
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            // This seal module mentions the forbidden name in string literals /
            // docs; skip it. Every other router source must stay clean.
            if path.file_name().and_then(|n| n.to_str()) == Some("plane.rs") {
                continue;
            }
            let text = fs::read_to_string(&path).expect("read rs");
            for (i, line) in text.lines().enumerate() {
                let t = line.trim();
                if t.starts_with("//") {
                    continue;
                }
                // Real imports / paths only — not prose in strings.
                let is_use = t.starts_with("use ") && t.contains("vecGradient");
                let is_extern = t.starts_with("extern crate vecGradient");
                let is_path = t.contains("vecGradient::");
                if is_use || is_extern || is_path {
                    offenders.push(format!("{}:{}: {}", path.display(), i + 1, t));
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "router src must not import vecGradient (would claim geometry without owning it):\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn clap_about_does_not_claim_geometry() {
        let cli = fs::read_to_string(manifest_dir().join("src/cli.rs")).expect("cli.rs");
        // Pull the #[command(...)] attribute block on Cli — not the whole file
        // (comments may mention the split).
        let start = cli
            .find("#[command(")
            .expect("Cli #[command] attribute");
        let end = cli[start..]
            .find("\npub struct Cli")
            .map(|i| start + i)
            .expect("pub struct Cli after #[command]");
        let attr = &cli[start..end];
        for needle in ["Vector15D", "Vector13D", "vecGradient", "geometry"] {
            assert!(
                !attr.contains(needle),
                "gradient-codec clap about must not claim {needle}; geometry CLI is {GEOMETRY_CLI_BIN}"
            );
        }
        assert!(
            attr.contains("orchestration") || attr.contains("budget"),
            "clap about should describe the router/orchestration plane"
        );
    }

    #[test]
    fn geometry_lives_in_vecgradient_member_with_v15d_bin() {
        let root = manifest_dir();
        let lib = root.join("vecGradient/src/lib.rs");
        let bin = root.join("vecGradient/src/bin/v15d.rs");
        assert!(
            lib.is_file(),
            "geometry steel missing: {}",
            lib.display()
        );
        assert!(
            bin.is_file(),
            "geometry CLI missing: {} (bin {GEOMETRY_CLI_BIN})",
            bin.display()
        );
        let lib_txt = fs::read_to_string(&lib).expect("lib.rs");
        assert!(
            lib_txt.contains("fn validate"),
            "vecGradient must expose validate() — steel surface for downstream cores"
        );
        let bin_txt = fs::read_to_string(&bin).expect("v15d.rs");
        assert!(
            bin_txt.contains("validate"),
            "v15d bin must call validate — geometry path, not router costume"
        );
        assert_eq!(GEOMETRY_PLANE_PACKAGE, "vecGradient");
        assert_eq!(GEOMETRY_CLI_BIN, "v15d");
        assert_eq!(ROUTER_PLANE, "orchestration");
    }

    #[test]
    fn readme_declares_plane_split_not_costume_couple() {
        let readme = fs::read_to_string(manifest_dir().join("README.md")).expect("README");
        assert!(
            readme.contains("## Plane split (P0.3)"),
            "README must declare the P0.3 plane split"
        );
        assert!(
            readme.contains("does not link `vecGradient`")
                || readme.contains("does not link vecGradient"),
            "README must state the router bin does not link vecGradient"
        );
        assert!(
            readme.contains("`v15d`") || readme.contains("bin `v15d`"),
            "README must name the geometry CLI v15d"
        );
    }
}
