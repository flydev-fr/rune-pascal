# Architecture

The public boundary is `src/bridge/include/rune_pas.h`. ChetCLI generates
`src/src/pascal/RunePas.pas`; RuneUi provides a small facade and RuneVcl owns the native
GPU host in a VCL HWND. The bridge handles generic nodes, native document flow,
themes, effects, selection, comboboxes and event delivery. It does not know about
a Markdown library, recent files, reader settings, translations or installer.

Document rendering accepts the bounded formatted fragment produced by an external
parser. Parsing files and choosing a Markdown dialect belong to the consuming app.

The reusable layer can be developed independently through its examples/tests.
Rune Markdown consumes this source tree using a Git submodule, sibling checkout or
explicit source-root override. Generic fixes belong here; reader-specific behavior
belongs to that application. The two projects version and package independently.

Rune and mORMot2 are clean upstream Git submodules at explicit commits. The Lab
and Settings library adaptations are in src/reference-lab and src/reference-settings;
no upstream entrypoint is modified. mORMot static artifacts are separate from its
source submodule and prepared through a hash-verified bundle.

The Markdown extension belongs to the reader application, not this SDK:
https://github.com/flydev-fr/mormot2-extensions/tree/main/mormot-markdown.
Its current upstream source directory is mormot-markdown/src.
