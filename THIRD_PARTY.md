# Third-party components and tools

| Component | License / attribution | Notice |
| --- | --- | --- |
| Rune | MIT, copyright 2026 Anomaly | vendor/rune/LICENSE |
| mORMot2 test framework | Arnaud Bouchez; existing MPL 1.1 / GPL 2.0 / LGPL 2.1 choice | vendor/mormot2/LICENCE.md |
| Inter / IBM Plex Mono | Existing font licenses retained | vendor/rune assets |
| ChetCLI | flydev-fr, based on Chet/Neslib.Clang by Erik van Bilsen | LICENSES/Chet-BSD-2-Clause.txt |

Upstream sources: https://github.com/anomalyco/rune,
https://github.com/synopse/mORMot2, https://github.com/flydev-fr/chet-cli,
https://github.com/neslib/Chet and https://github.com/neslib/Neslib.Clang.
ChetCLI/libclang are build tools; libclang is part of LLVM, under Apache-2.0
with LLVM exceptions. Generated binding attribution is restored by the script.

Exact upstream submodule commits are recorded in vendor/SOURCES.json and the Gitlinks. The mORMot2
reference commit is pinned there. Static binary provenance/hashes are retained
separately under support/mormot2-static. Cargo.lock
pins Rust dependencies. prepare-legal.ps1 collects their available license texts
and selected Windows dependency inventory into build/legal for SDK/app packaging.

The reference adapter entrypoints derive from upstream Rune examples and retain
Anomaly's MIT copyright alongside flydev-fr's library-adapter modifications.
No upstream code is patched inside a submodule.
