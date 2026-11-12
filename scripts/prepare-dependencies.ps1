# SPDX-License-Identifier: MIT
# Copyright (c) 2026 flydev-fr.
[CmdletBinding()]
param([switch]$Offline)
$ErrorActionPreference='Stop';$root=Split-Path -Parent $PSScriptRoot
if(-not(Test-Path -LiteralPath "$root\vendor\rune\crates\rune\Cargo.toml") -or -not(Test-Path -LiteralPath "$root\vendor\mormot2\src\mormot.defines.inc")){
 if($Offline){throw 'Submodules are missing. Run git submodule update --init --recursive before an offline build.'}
 git -C $root submodule update --init --recursive
 if($LASTEXITCODE -ne 0){throw 'Submodule initialization failed'}
}
& "$root\scripts\prepare-mormot-static.ps1"
