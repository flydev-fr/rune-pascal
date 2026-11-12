# Rune Pascal

Native Rune UI integration for Delphi and Free Pascal. This is the development
repository for the Rust/C bridge, generated Pascal bindings, UI facade and VCL host.
The standalone **Rune Markdown** application belongs in its own `rune-markdown`
repository; it consumes this SDK as a source dependency.

## Clone

```powershell
git clone --recurse-submodules https://github.com/flydev-fr/rune-pascal.git
cd rune-pascal
```

For an existing checkout, run `git submodule update --init --recursive`.
`vendor/rune` and `vendor/mormot2` are real Git submodules pinned to tested commits,
not copied source trees. Their revisions are also listed in vendor/SOURCES.json.
The build prepares the matching bundled mORMot static-link inputs automatically.

## Develop

Windows x64, Delphi 13, Rust (pinned by rust-toolchain.toml), MSVC build tools and a
compatible GPU driver are required for the complete build and native tests.

```powershell
.\build.ps1 -Test -Examples
.\build\bin\TodoPas.exe
.\build\bin\RuneVclDemo.exe
```

Open any `examples/pascal/*.dproj` directly in Delphi to develop a Pascal example.
The project search paths refer to `src/pascal` in this checkout. Build the DLL first
and keep it beside the EXE. Open this repository's Cargo.toml for Rust development.
No reader project, installer or application profile is needed to build the SDK.

- `-BridgeOnly`: build only the DLL (and Rust tests with `-Test`).
- `-Examples`: compile Settings, Lab, Todo, shader buttons and VCL embedding demos.
- `-Test`: run Rust tests plus the Pascal API, Lab, UI and native VCL suites.
- `-SkipRust`: reuse the optimized DLL in the selected Cargo cache.
- `-RustTargetDir`, `-Offline`, `-DelphiRoot`: override build environment choices.

## Source layout

| Path | Responsibility |
| --- | --- |
| `src/bridge` | Rust library, C ABI and generic native document rendering |
| `src/reference-lab`, `src/reference-settings` | Library adapters for upstream reference demos |
| `src/pascal` | Generated RunePas, RuneUi facade, RuneVcl host |
| `examples/pascal` | Independent runnable Pascal examples |
| `tests/pascal` | mORMot-based SDK regression suites |
| `vendor/rune` | Pinned upstream Rune Git submodule, unchanged |
| `vendor/mormot2` | Pinned test-framework Git submodule; not required by the SDK units |
| `support/mormot2-static` | Exact tested binary link inputs and SHA-256 manifest |
| `scripts` | Bindings, Delphi builds, notices and SDK packaging |
| `build`, `packages` | Ignored outputs, never source dependencies |

## Documentation and packages

See [SDK usage](docs/GETTING_STARTED.md), [VCL](docs/VCL.md),
[build details](docs/BUILD.md) and [architecture](docs/ARCHITECTURE.md).

```powershell
.\scripts\regenerate-bindings.ps1 -Chet 'path\ChetCLI.exe'
.\scripts\check-vendor.ps1
.\scripts\package-sdk.ps1 -SkipBuild
```

The SDK ZIP is a convenience for binary consumers. Source consumers, including
Rune Markdown, compile against this repository rather than copying generated units.
A complete VCL widget palette is not yet delivered; see the VCL limitations.

Project-owned code is MIT, copyright 2026 flydev-fr. Preserve upstream notices.
Bindings credit [flydev-fr/chet-cli](https://github.com/flydev-fr/chet-cli) and
Erik van Bilsen's original Chet/Neslib.Clang. See [licensing](LICENSE.md).
