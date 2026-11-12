# SPDX-License-Identifier: MIT
# Copyright (c) 2026 flydev-fr.
[CmdletBinding()]
param()
$ErrorActionPreference='Stop';$root=Split-Path -Parent $PSScriptRoot
$manifest=Get-Content -LiteralPath "$root\vendor\SOURCES.json" -Raw|ConvertFrom-Json
foreach($property in $manifest.sources.PSObject.Properties){
 $name=$property.Name;$spec=$property.Value;$path=Join-Path $root ('vendor\'+$name)
 $head=git -C $path rev-parse HEAD;if($LASTEXITCODE -ne 0){throw "Submodule missing: $name"}
 if($head -ne $spec.commit){throw "Submodule revision differs: $name ($head)"}
 $indexed=git -C $root ls-files --stage -- ('vendor/'+$name)
 if($indexed -notmatch ('^160000 '+[regex]::Escape($spec.commit)+' ')){throw "Gitlink not pinned to the expected revision: $name"}
 $changes=git -C $path status --porcelain --untracked-files=normal
 if($changes){throw "Upstream submodule contains local changes: $name`n$changes"}
 Write-Host "${name}: $head (clean, pinned Git submodule)"
}
$bundle=Join-Path $root 'support\mormot2-static'
$statics=Get-Content -LiteralPath "$bundle\FILES.json" -Raw|ConvertFrom-Json
foreach($entry in $statics.files){if((Get-FileHash -LiteralPath (Join-Path $bundle $entry.path)).Hash -ne $entry.sha256){throw "Static bundle changed: $($entry.path)"}}
Write-Host "Static artifact bundle verified: $($statics.files.Count) files."
