# SPDX-License-Identifier: MIT
# Copyright (c) 2026 flydev-fr.
[CmdletBinding()]
param()
$ErrorActionPreference='Stop';$root=Split-Path -Parent $PSScriptRoot
$bundle=Join-Path $root 'support\mormot2-static'
$destination=[IO.Path]::GetFullPath((Join-Path $root 'vendor\mormot2\static'))
if(-not(Test-Path -LiteralPath "$root\vendor\mormot2\src\mormot.defines.inc")){throw 'Initialize the mORMot2 submodule first.'}
$manifest=Get-Content -LiteralPath "$bundle\FILES.json" -Raw|ConvertFrom-Json
foreach($entry in $manifest.files){
 $source=[IO.Path]::GetFullPath((Join-Path $bundle $entry.path));$target=[IO.Path]::GetFullPath((Join-Path $destination $entry.path))
 if(-not $source.StartsWith($bundle+'\',[StringComparison]::OrdinalIgnoreCase) -or -not $target.StartsWith($destination+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Static file path outside bundle'}
 if((Get-FileHash -LiteralPath $source).Hash -ne $entry.sha256){throw "Static bundle changed: $($entry.path)"}
 if(Test-Path -LiteralPath $target){
  if((Get-FileHash -LiteralPath $target).Hash -ne $entry.sha256){throw "Different static binary already exists: $target"}
 }else{
  New-Item -ItemType Directory -Force -Path (Split-Path -Parent $target)|Out-Null
  Copy-Item -LiteralPath $source -Destination $target
 }
}
Write-Host "mORMot2 static inputs ready: $($manifest.files.Count) verified files."
