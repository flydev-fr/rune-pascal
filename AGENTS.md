# Rune Pascal

- Reusable bridge/SDK only. Reader application belongs in the separate rune-markdown repository.
- Rust: src/bridge; Pascal SDK: src/pascal; examples and tests: their own directories.
- RunePas.pas is generated: use scripts/regenerate-bindings.ps1 and preserve both
  flydev-fr/chet-cli and Erik van Bilsen's Chet/Neslib.Clang credits.
- MIT/flydev-fr headers for project-owned code; preserve upstream notices.
- Use native Rune controls and document flow. Keep application state outside the bridge.
- English docs/comments/messages. No DFM; define VCL UI in Pascal units.
- build.ps1 -Test -Examples checks the SDK. mORMot tests use /noenter.
- Vendor sources are clean pinned Git submodules. Keep adapters outside vendor/.
- Update Gitlinks and vendor/SOURCES.json together; run scripts/check-vendor.ps1.
- Preserve hash-verified static artifacts under support/mormot2-static.
- Conventional Commits, no AI trailers. Do not commit build/, packages/ or local profiles.
