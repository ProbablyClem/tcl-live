use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../front/src");

    let status = Command::new("wasm-pack")
        .args([
            "build",
            "../front",
            "--target",
            "web",
            "--out-dir",
            "../static/pkg",
        ])
        .status()
        .expect("failed to run wasm-pack");

    if !status.success() {
        panic!("wasm build failed");
    }
}
