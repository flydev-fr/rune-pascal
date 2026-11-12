# SPDX-License-Identifier: MIT
# Copyright (c) 2026 flydev-fr.
[CmdletBinding()]
param([switch]$SkipBuild)
$ErrorActionPreference='Stop';$root=Split-Path -Parent $PSScriptRoot
if(-not $SkipBuild){& "$root\build.ps1" -Test}
& "$root\scripts\prepare-legal.ps1"
$version=(Get-Content -LiteralPath "$root\VERSION" -Raw).Trim()
$stage=Join-Path $root "build\package\Rune-Pascal-$version-windows-x64"
New-Item -ItemType Directory -Force -Path $stage|Out-Null
Copy-Item -LiteralPath "$root\src\pascal" -Destination $stage -Recurse -Force
Copy-Item -LiteralPath "$root\src\bridge\include" -Destination $stage -Recurse -Force
Copy-Item -LiteralPath "$root\docs" -Destination $stage -Recurse -Force
Copy-Item -LiteralPath "$root\build\legal" -Destination $stage -Recurse -Force
Copy-Item -LiteralPath "$root\build\bin\rune_pas.dll" -Destination $stage -Force
Copy-Item -LiteralPath "$root\LICENSE" -Destination $stage -Force
$packages=Join-Path $root 'packages';New-Item -ItemType Directory -Force -Path $packages|Out-Null
$zip=Join-Path $packages "Rune-Pascal-$version-windows-x64.zip"
Get-ChildItem -LiteralPath $stage -Recurse -File | Where-Object {$_.LastWriteTime.Year -lt 1980} | ForEach-Object {$_.LastWriteTime=[datetime]'1980-01-01'}
Compress-Archive -Path $stage -DestinationPath $zip -Force
Write-Host $zip
