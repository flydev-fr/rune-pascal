# Validation of the separated SDK

Local Windows x64, Delphi 13.2 and Rust 1.99.1. The standalone build passed
22 Rust tests and 449 Pascal assertions (API 17, Lab 339, generic UI 71,
native VCL 22). All five Pascal examples compiled from their new source paths.
The SDK build does not access application sources or the earlier combined tree.
An existing Cargo cache was reused only for compiled dependencies.

The Rune/test-framework Gitlinks and clean submodule worktrees passed
vendor/SOURCES.json verification, including the static artifact bundle.
The independent SDK ZIP was built with Pascal sources, C headers, DLL, documentation,
project licensing and notices for 173 selected Windows Rust dependencies.
The local Git repository is initialized with origin flydev-fr/rune-pascal.
Rune is pinned at a7f276c3630f85a06a0230295da4d2bd172d8d3b; mORMot2 at
fc4c033da5520d25221025efb78fcc4ea55a55a0. Both upstream worktrees remain clean.
The conversion passed the same 22 Rust tests, 449 Pascal assertions and five
example builds. The generated Lab compositor glue only resolves module locations;
no upstream source file is patched. Original vendor snapshots were backed up under
ignored build/ before replacement. No commits or pushes were performed.
