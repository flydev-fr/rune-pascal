# mORMot2 static-link inputs

These are the exact binary inputs used by the already-tested Delphi/FPC builds.
They are build artifacts, not a second copy of framework sources. Their original
mORMot2 license applies; see ../../vendor/mormot2/LICENCE.md and static/README.md
in that submodule. FILES.json records exact hashes and the available provenance.

scripts/prepare-mormot-static.ps1 installs them into the submodule static directory
before Pascal builds. Upstream ignores these generated/binary paths; no tracked
submodule source is changed. No editor history or compiler-generated core imports
are retained. Upstream artifact retrieval can replace this bundle in a future
explicit dependency update; do not silently substitute current server binaries.
