//! Build script: compiles the `front` crate to wasm and runs `wasm-bindgen`
//! against it so the resulting JS/WASM bundle lands in `static/pkg/`.
//!
//! Critical detail: the inner `cargo build` invoked here MUST use a
//! `CARGO_TARGET_DIR` distinct from the outer cargo's target dir, otherwise
//! both cargos try to acquire the FLOCK on `target/debug/.cargo-lock` and
//! deadlock (outer cargo holds it; inner cargo waits forever for it; outer
//! cargo waits for the build script to finish). We use `target-front/`.
//!
//! Two paths:
//!   - dev (default): direct `cargo build` + `wasm-bindgen` with the
//!     `wasm-dev` profile. Skips the `wasm-pack` wrapper, skips DWARF
//!     generation, and short-circuits when output is already up-to-date.
//!   - release: delegate to `wasm-pack` so we keep `wasm-opt` size
//!     optimization (assuming `wasm-pack` finds/installs it). Release also
//!     uses a separate target dir to avoid the deadlock.
//!
//! Set `SKIP_WASM_BUILD=1` to skip the front build entirely (useful when
//! iterating on `back/` only).

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;

fn main() {
    println!("cargo:rerun-if-env-changed=SKIP_WASM_BUILD");
    println!("cargo:rerun-if-changed=../front/src");
    println!("cargo:rerun-if-changed=../front/Cargo.toml");
    println!("cargo:rerun-if-changed=../Cargo.lock");

    if std::env::var_os("SKIP_WASM_BUILD").is_some() {
        println!("cargo:warning=SKIP_WASM_BUILD set; not rebuilding front wasm");
        return;
    }

    let workspace_root = workspace_root();
    let profile = build_profile();

    if profile == "release" {
        run_wasm_pack_release(&workspace_root);
    } else {
        run_dev_wasm_build(&workspace_root);
    }
}

fn run_dev_wasm_build(workspace_root: &Path) {
    let pkg_dir = workspace_root.join("static/pkg");
    // Distinct target dir for the front so the inner cargo doesn't deadlock
    // on the outer `cargo run`'s target/debug/.cargo-lock.
    let front_target_dir = workspace_root.join("target-front");
    let wasm_artifact = front_target_dir
        .join("wasm32-unknown-unknown")
        .join("wasm-dev")
        .join("front.wasm");

    cargo_build_front(workspace_root, &front_target_dir);

    let bindgen_out = pkg_dir.join("front_bg.wasm");
    if is_up_to_date(&bindgen_out, &wasm_artifact) {
        return;
    }

    run_wasm_bindgen(&wasm_artifact, &pkg_dir);
}

fn cargo_build_front(workspace_root: &Path, front_target_dir: &Path) {
    let mut cmd = Command::new(cargo_bin());
    cmd.current_dir(workspace_root)
        .args([
            "build",
            "-p",
            "front",
            "--lib",
            "--target",
            "wasm32-unknown-unknown",
            "--profile",
            "wasm-dev",
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Cargo passes a swarm of CARGO_* env vars into build scripts. If we leave
    // them in place, the inner cargo invocation gets confused (wrong manifest
    // dir, wrong feature set, target-dir tied to the outer build). Strip them
    // so the inner cargo behaves as if invoked from the shell.
    for (key, _) in std::env::vars_os() {
        let key_bytes = key.as_encoded_bytes();
        if key_bytes.starts_with(b"CARGO_") || key_bytes == b"RUSTFLAGS" {
            cmd.env_remove(&key);
        }
    }
    // CRITICAL: separate target dir to avoid FLOCK deadlock on
    // target/debug/.cargo-lock.
    cmd.env("CARGO_TARGET_DIR", front_target_dir);

    let status = cmd
        .status()
        .expect("failed to spawn `cargo build` for front wasm");
    if !status.success() {
        panic!("front wasm cargo build failed");
    }
}

fn run_wasm_bindgen(wasm_artifact: &Path, pkg_dir: &Path) {
    std::fs::create_dir_all(pkg_dir).expect("failed to create static/pkg");

    let bindgen = locate_wasm_bindgen();
    let status = Command::new(&bindgen)
        .args([
            "--target",
            "web",
            "--out-name",
            "front",
            "--no-typescript",
            "--out-dir",
        ])
        .arg(pkg_dir)
        .arg(wasm_artifact)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap_or_else(|e| panic!("failed to spawn wasm-bindgen ({}): {e}", bindgen.display()));
    if !status.success() {
        panic!("wasm-bindgen failed");
    }
}

/// Locates `wasm-bindgen`. Preference order:
///   1. `WASM_BINDGEN` env override.
///   2. wasm-pack's cached binary at `~/.cache/.wasm-pack/wasm-bindgen-*/wasm-bindgen`
///      (whatever version was last fetched matches the crate version because
///      `wasm-pack` enforces that).
///   3. `wasm-bindgen` on PATH.
fn locate_wasm_bindgen() -> PathBuf {
    if let Some(p) = std::env::var_os("WASM_BINDGEN") {
        return PathBuf::from(p);
    }
    if let Some(home) = std::env::var_os("HOME") {
        let cache = PathBuf::from(home).join(".cache/.wasm-pack");
        if let Ok(entries) = std::fs::read_dir(&cache) {
            for e in entries.flatten() {
                let candidate = e.path().join("wasm-bindgen");
                if candidate.is_file() {
                    return candidate;
                }
            }
        }
    }
    PathBuf::from("wasm-bindgen")
}

fn run_wasm_pack_release(workspace_root: &Path) {
    let front_dir = workspace_root.join("front");
    let out_dir = workspace_root.join("static/pkg");
    let front_target_dir = workspace_root.join("target-front");

    let mut cmd = Command::new("wasm-pack");
    cmd.args(["build"])
        .arg(&front_dir)
        .args(["--target", "web", "--out-dir"])
        .arg(&out_dir)
        .arg("--release")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Same anti-deadlock measure as the dev path.
    for (key, _) in std::env::vars_os() {
        let key_bytes = key.as_encoded_bytes();
        if key_bytes.starts_with(b"CARGO_") || key_bytes == b"RUSTFLAGS" {
            cmd.env_remove(&key);
        }
    }
    cmd.env("CARGO_TARGET_DIR", &front_target_dir);

    let status = cmd.status().expect("failed to run wasm-pack");
    if !status.success() {
        panic!("wasm-pack release build failed");
    }
}

fn is_up_to_date(output: &Path, input: &Path) -> bool {
    let (Ok(out_meta), Ok(in_meta)) = (std::fs::metadata(output), std::fs::metadata(input)) else {
        return false;
    };
    let (Ok(out_t), Ok(in_t)) = (out_meta.modified(), in_meta.modified()) else {
        return false;
    };
    out_t >= in_t && out_t > SystemTime::UNIX_EPOCH
}

fn workspace_root() -> PathBuf {
    // back/ is a workspace member, so the workspace root is the parent of
    // CARGO_MANIFEST_DIR.
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest)
        .parent()
        .expect("back/ should have a parent (workspace root)")
        .to_path_buf()
}

fn cargo_bin() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"))
}

fn build_profile() -> String {
    // OUT_DIR ends with `target/<profile>/build/<crate>-<hash>/out`, so the
    // profile is the 3rd-from-last component.
    std::env::var("OUT_DIR")
        .ok()
        .and_then(|s| {
            PathBuf::from(s)
                .components()
                .rev()
                .nth(3)
                .and_then(|c| c.as_os_str().to_str().map(String::from))
        })
        .unwrap_or_else(|| "debug".to_string())
}
