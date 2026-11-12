# Build and tests

Run `build.ps1 -Test -Examples` from the repository root. The Rust workspace builds
`src/bridge` and two reference-demo adapters, depending on the upstream Rune
workspace inside its pinned submodule. `src/pascal` is the only Pascal SDK
source location. SDK builds do not read a sibling application repository.

Outputs are `build/bin`, `build/dcu` and, by default, `build/cargo`. An alternate
Cargo cache can be selected with `-RustTargetDir`; it supplies compiled artifacts,
not source code. `-Offline` requires an already populated Cargo registry.

Delphi is discovered from BDS, the registry or the standard installation path.
Use `-DelphiRoot` to override discovery. `scripts/invoke-delphi.ps1` can compile
any example/test `.dproj` directly. All project search paths are repository-relative.
FPC-compatible examples require FPC 3.2.3 or later; the VCL host requires Delphi.

Tests use mORMot2 and `/noenter`; the four Pascal suites include native GPU windows.
Clipboard writes are intercepted by the VCL regression test to preserve the user's
clipboard. A Windows desktop session and compatible GPU are required.

Bindings must be regenerated with `scripts/regenerate-bindings.ps1`, which runs
ChetCLI, validates its configuration, then restores generator attribution. Do not
hand-edit the generated RunePas declarations. The reference .chet configuration
is `src/pascal/rune_pas.chet`.

`check-vendor.ps1` verifies the two Gitlink revisions, clean upstream worktrees
and the static artifact bundle. `prepare-dependencies.ps1` initializes missing
submodules and installs the hash-verified static inputs; offline builds require
initialized submodules. `vendor/SOURCES.json` records exact upstream commits. `package-sdk.ps1` collects notices and packages the DLL,
Pascal units, C headers and SDK documentation. It does not package the reader.

## Dependency maintenance

Keep submodules detached at the tested commits. To update, fetch the upstream,
checkout an explicit commit, review/adjust the adapters, update SOURCES.json,
run the full build/tests, then stage the Gitlink and manifest together. Do not
commit generated files or local source patches inside the upstream submodules.

The SDK exposes upstream Lab/Settings through adapter crates outside vendor/.
Their adapted entrypoints retain Anomaly's copyright and MIT provenance; Lab's
unchanged supporting modules and font assets are referenced from the submodule.
The original upstream main.rs files remain intact and independently runnable.

mORMot does not track its static-link binaries. The matching previously tested
inputs live in support/mormot2-static, with exact SHA-256 hashes and provenance.
The preparation script refuses to overwrite different installed binaries.
