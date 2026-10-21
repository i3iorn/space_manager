//! The dependency-direction check.
//!
//! The architecture rests on one rule: **dependencies point one way.**
//!
//! ```text
//! space_manager  →  ui  →  sim  →  domain
//! ```
//!
//! `domain` depends on nothing of ours. `sim` may use `domain` but never `ui`.
//! `ui` may use both. The rule is easy to state and easy to break by accident
//! — one convenience import is enough — so it is checked here, and a violation
//! fails the build.
//!
//! The check reads `cargo metadata`, so it sees the *resolved* dependency graph
//! rather than the intent expressed in the manifests.
//!
//! See `docs/ARCHITECTURE.md` §3.

use std::collections::BTreeMap;
use std::process::Command;

/// The crates in this workspace, and what each of them is allowed to depend on.
const ALLOWED: &[(&str, &[&str])] = &[
    ("domain", &[]),
    ("sim", &["domain"]),
    ("ui", &["domain", "sim"]),
    ("space_manager", &["domain", "sim", "ui"]),
];

/// The workspace members, and the local crates each one depends on.
fn local_dependencies() -> BTreeMap<String, BTreeMap<String, String>> {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("the crate lives two directories below the workspace root")
        .to_path_buf();

    let output = Command::new(cargo_binary())
        .arg("metadata")
        .arg("--no-deps")
        .arg("--format-version")
        .arg("1")
        .current_dir(&workspace_root)
        .output()
        .expect("cargo metadata runs");

    assert!(
        output.status.success(),
        "cargo metadata failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("cargo metadata is valid JSON");

    let members: std::collections::BTreeSet<String> = metadata["workspace_members"]
        .as_array()
        .expect("workspace_members is an array")
        .iter()
        .filter_map(|member| member.as_str().map(str::to_owned))
        .collect();

    let mut result: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    for package in metadata["packages"]
        .as_array()
        .expect("packages is an array")
    {
        let name = package["name"].as_str().expect("a package has a name");
        let id = package["id"].as_str().expect("a package has an id");
        if !members.contains(id) {
            continue;
        }

        let mut dependencies = BTreeMap::new();
        for dependency in package["dependencies"]
            .as_array()
            .expect("dependencies is an array")
        {
            let dependency_name = dependency["name"]
                .as_str()
                .expect("a dependency has a name");
            if members
                .iter()
                .any(|member| member.ends_with(&format!("#{dependency_name}@")))
            {
                let kind = dependency["kind"]
                    .as_str()
                    .expect("a dependency has a kind")
                    .to_owned();
                dependencies.insert(dependency_name.to_owned(), kind);
            }
        }
        result.insert(name.to_owned(), dependencies);
    }

    result
}

/// The cargo executable to use.
fn cargo_binary() -> std::path::PathBuf {
    // `CARGO` is set by cargo itself when it runs a test, and points at the
    // same binary that is running the test. It is more reliable than assuming
    // "cargo" is on the PATH.
    std::env::var_os("CARGO").map_or_else(
        || std::path::PathBuf::from("cargo"),
        std::path::PathBuf::from,
    )
}

#[test]
fn the_dependency_direction_holds() {
    let graph = local_dependencies();

    for (name, allowed) in ALLOWED {
        let dependencies = graph
            .get(*name)
            .unwrap_or_else(|| panic!("the workspace has no crate called {name}"));

        for dependency in dependencies.keys() {
            assert!(
                allowed.contains(&dependency.as_str()),
                "{name} must not depend on {dependency}. \
                 The dependency direction is domain ← sim ← ui ← space_manager; \
                 see docs/ARCHITECTURE.md §3."
            );
        }
    }
}

#[test]
fn every_workspace_crate_is_listed() {
    let graph = local_dependencies();
    for (name, _) in ALLOWED {
        assert!(
            graph.contains_key(*name),
            "the crate {name} is missing from the workspace"
        );
    }
}

#[test]
fn the_domain_crate_depends_on_nothing_of_ours() {
    let graph = local_dependencies();
    let dependencies = graph.get("domain").expect("the domain crate exists");
    assert!(
        dependencies.is_empty(),
        "the domain crate must depend on nothing of ours, but it depends on {dependencies:?}"
    );
}
