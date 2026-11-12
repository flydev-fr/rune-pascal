// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
// Preserve upstream gfx source while making its nested module locations explicit.
use std::{env, fs, path::PathBuf};
fn main() {
    let upstream = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../../vendor/rune/examples/lab/src").canonicalize().unwrap();
    let source = upstream.join("gfx.rs");
    println!("cargo:rerun-if-changed={}", source.display());
    let mut code = fs::read_to_string(&source).unwrap();
    // include! is inside an already declared module, so use ordinary doc comments.
    code = code.lines().map(|line| line.strip_prefix("//!").map_or_else(
        || line.to_owned(), |doc| format!("//{doc}"))).collect::<Vec<_>>().join("\n");
    for name in ["gpu", "scenes"] {
        let target = upstream.join("gfx").join(format!("{name}.rs"));
        let path = target.to_string_lossy().replace('\\', "/");
        code = code.replace(&format!("mod {name};"), &format!("#[path = {path:?}] mod {name};"));
    }
    fs::write(PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("gfx.rs"), code).unwrap();
}
